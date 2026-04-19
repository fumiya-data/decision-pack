use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::algorithm::{CandidateScore, Weights, build_scores};
use crate::db::{CustomerRecord, ItemRecord, OrderItemHistoryRow};

#[derive(Debug, Clone, PartialEq)]
pub struct MetricSummary {
    pub hit_at_3: f64,
    pub hit_at_5: f64,
    pub recall_at_5: f64,
    pub recall_at_10: f64,
    pub ndcg_at_5: f64,
    pub ndcg_at_10: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationReport {
    pub eligible_customers: usize,
    pub customers_with_predictions: usize,
    pub model: MetricSummary,
    pub popularity_baseline: MetricSummary,
}

pub fn evaluate_recommender(
    customers: &[CustomerRecord],
    items: &[ItemRecord],
    history: &[OrderItemHistoryRow],
    top_n: usize,
    weights: Weights,
    min_orders_for_eval: usize,
) -> EvaluationReport {
    let split = split_history(history, min_orders_for_eval);
    let (model_scores, _) = build_scores(
        customers,
        items,
        &split.training_history,
        top_n.max(10),
        weights,
    );
    let prediction_map = score_map(&model_scores);
    let popularity = global_popularity(&split.training_history, top_n.max(10));

    let model = aggregate_metrics(&split.targets, &prediction_map);
    let baseline = aggregate_metrics(
        &split.targets,
        &split
            .targets
            .keys()
            .map(|customer_id| (customer_id.clone(), popularity.clone()))
            .collect::<HashMap<_, _>>(),
    );

    EvaluationReport {
        eligible_customers: split.targets.len(),
        customers_with_predictions: split
            .targets
            .keys()
            .filter(|customer_id| prediction_map.contains_key(*customer_id))
            .count(),
        model,
        popularity_baseline: baseline,
    }
}

fn score_map(scores: &[CandidateScore]) -> HashMap<String, Vec<String>> {
    let mut by_customer = BTreeMap::<String, Vec<(i32, f64, String)>>::new();
    for row in scores {
        by_customer
            .entry(row.customer_id.clone())
            .or_default()
            .push((row.rank, row.score, row.item_id.clone()));
    }

    by_customer
        .into_iter()
        .map(|(customer_id, mut rows)| {
            rows.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.total_cmp(&a.1)));
            (
                customer_id,
                rows.into_iter().map(|(_, _, item_id)| item_id).collect(),
            )
        })
        .collect()
}

fn split_history(history: &[OrderItemHistoryRow], min_orders_for_eval: usize) -> SplitResult {
    let mut by_customer = BTreeMap::<String, Vec<&OrderItemHistoryRow>>::new();
    for row in history {
        by_customer
            .entry(row.customer_id.clone())
            .or_default()
            .push(row);
    }
    for rows in by_customer.values_mut() {
        rows.sort_by(|a, b| {
            a.ordered_at
                .cmp(&b.ordered_at)
                .then(a.order_id.cmp(&b.order_id))
                .then(a.item_id.cmp(&b.item_id))
        });
    }

    let mut training_history = Vec::new();
    let mut targets = HashMap::<String, BTreeSet<String>>::new();

    for (customer_id, rows) in by_customer {
        let mut order_sequence = rows
            .iter()
            .map(|row| row.order_id.clone())
            .collect::<Vec<_>>();
        order_sequence.dedup();
        if order_sequence.len() < min_orders_for_eval {
            training_history.extend(rows.into_iter().cloned());
            continue;
        }

        let holdout_order_id = order_sequence.last().cloned().unwrap_or_default();
        let mut holdout_items = BTreeSet::new();
        for row in rows {
            if row.order_id == holdout_order_id {
                holdout_items.insert(row.item_id.clone());
            } else {
                training_history.push(row.clone());
            }
        }
        if !holdout_items.is_empty() {
            targets.insert(customer_id, holdout_items);
        }
    }

    SplitResult {
        training_history,
        targets,
    }
}

fn global_popularity(history: &[OrderItemHistoryRow], top_n: usize) -> Vec<String> {
    let mut counts = HashMap::<String, i32>::new();
    for row in history {
        *counts.entry(row.item_id.clone()).or_insert(0) += row.quantity.max(0);
    }
    let mut ranked = counts.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    ranked
        .into_iter()
        .take(top_n.max(10))
        .map(|(item_id, _)| item_id)
        .collect()
}

fn aggregate_metrics(
    targets: &HashMap<String, BTreeSet<String>>,
    predictions: &HashMap<String, Vec<String>>,
) -> MetricSummary {
    let count = targets.len().max(1) as f64;
    let mut hit_at_3 = 0.0;
    let mut hit_at_5 = 0.0;
    let mut recall_at_5 = 0.0;
    let mut recall_at_10 = 0.0;
    let mut ndcg_at_5 = 0.0;
    let mut ndcg_at_10 = 0.0;

    for (customer_id, actual) in targets {
        let predicted = predictions.get(customer_id).cloned().unwrap_or_default();
        hit_at_3 += hit_at_k(&predicted, actual, 3);
        hit_at_5 += hit_at_k(&predicted, actual, 5);
        recall_at_5 += recall_at_k(&predicted, actual, 5);
        recall_at_10 += recall_at_k(&predicted, actual, 10);
        ndcg_at_5 += ndcg_at_k(&predicted, actual, 5);
        ndcg_at_10 += ndcg_at_k(&predicted, actual, 10);
    }

    MetricSummary {
        hit_at_3: hit_at_3 / count,
        hit_at_5: hit_at_5 / count,
        recall_at_5: recall_at_5 / count,
        recall_at_10: recall_at_10 / count,
        ndcg_at_5: ndcg_at_5 / count,
        ndcg_at_10: ndcg_at_10 / count,
    }
}

fn hit_at_k(predicted: &[String], actual: &BTreeSet<String>, k: usize) -> f64 {
    f64::from(
        predicted
            .iter()
            .take(k)
            .any(|item_id| actual.contains(item_id)),
    )
}

fn recall_at_k(predicted: &[String], actual: &BTreeSet<String>, k: usize) -> f64 {
    if actual.is_empty() {
        return 0.0;
    }
    let hits = predicted
        .iter()
        .take(k)
        .filter(|item_id| actual.contains(*item_id))
        .count();
    hits as f64 / actual.len() as f64
}

fn ndcg_at_k(predicted: &[String], actual: &BTreeSet<String>, k: usize) -> f64 {
    let dcg = predicted
        .iter()
        .take(k)
        .enumerate()
        .filter(|(_, item_id)| actual.contains(*item_id))
        .map(|(idx, _)| 1.0 / ((idx + 2) as f64).log2())
        .sum::<f64>();
    let ideal_len = actual.len().min(k);
    if ideal_len == 0 {
        return 0.0;
    }
    let idcg = (0..ideal_len)
        .map(|idx| 1.0 / ((idx + 2) as f64).log2())
        .sum::<f64>();
    if idcg == 0.0 { 0.0 } else { dcg / idcg }
}

#[derive(Debug, Clone)]
struct SplitResult {
    training_history: Vec<OrderItemHistoryRow>,
    targets: HashMap<String, BTreeSet<String>>,
}

#[cfg(test)]
mod tests {
    use super::evaluate_recommender;
    use crate::algorithm::Weights;
    use crate::db::{CustomerRecord, ItemRecord, OrderItemHistoryRow};
    use chrono::{Duration, Utc};

    #[test]
    fn evaluation_report_has_non_negative_metrics() {
        let now = Utc::now();
        let customers = vec![
            CustomerRecord {
                customer_id: "C1".into(),
                country: Some("Japan".into()),
                status: Some("active".into()),
                tier: Some("gold".into()),
            },
            CustomerRecord {
                customer_id: "C2".into(),
                country: Some("Japan".into()),
                status: Some("active".into()),
                tier: Some("silver".into()),
            },
        ];
        let items = vec![
            ItemRecord {
                item_id: "A".into(),
                item_name: "A".into(),
                category: "cat".into(),
                is_active: true,
            },
            ItemRecord {
                item_id: "B".into(),
                item_name: "B".into(),
                category: "cat".into(),
                is_active: true,
            },
            ItemRecord {
                item_id: "C".into(),
                item_name: "C".into(),
                category: "cat".into(),
                is_active: true,
            },
        ];
        let history = vec![
            OrderItemHistoryRow {
                customer_id: "C1".into(),
                order_id: "O1".into(),
                ordered_at: Some(now - Duration::days(10)),
                item_id: "A".into(),
                quantity: 1,
            },
            OrderItemHistoryRow {
                customer_id: "C1".into(),
                order_id: "O2".into(),
                ordered_at: Some(now - Duration::days(3)),
                item_id: "B".into(),
                quantity: 1,
            },
            OrderItemHistoryRow {
                customer_id: "C2".into(),
                order_id: "O3".into(),
                ordered_at: Some(now - Duration::days(8)),
                item_id: "A".into(),
                quantity: 1,
            },
            OrderItemHistoryRow {
                customer_id: "C2".into(),
                order_id: "O4".into(),
                ordered_at: Some(now - Duration::days(2)),
                item_id: "C".into(),
                quantity: 1,
            },
        ];

        let report = evaluate_recommender(
            &customers,
            &items,
            &history,
            5,
            Weights {
                repeat: 0.5,
                transition: 0.3,
                segment: 0.2,
            },
            2,
        );

        assert_eq!(report.eligible_customers, 2);
        assert!(report.model.hit_at_5 >= 0.0);
        assert!(report.popularity_baseline.ndcg_at_10 >= 0.0);
    }
}
