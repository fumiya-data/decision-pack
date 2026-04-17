use chrono::Utc;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::db::{CustomerRecord, ItemRecord, OrderItemHistoryRow};

#[derive(Debug, Clone)]
pub struct CandidateScore {
    pub customer_id: String,
    pub item_id: String,
    pub score: f64,
    pub rank: i32,
}

#[derive(Debug, Clone)]
pub struct DemandForecastRow {
    pub item_id: String,
    pub expected_qty: i32,
    pub low_qty: i32,
    pub high_qty: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct Weights {
    pub repeat: f64,
    pub transition: f64,
    pub segment: f64,
}

pub fn build_scores(
    customers: &[CustomerRecord],
    items: &[ItemRecord],
    history: &[OrderItemHistoryRow],
    top_n: usize,
    weights: Weights,
) -> (Vec<CandidateScore>, Vec<DemandForecastRow>) {
    let active_items: Vec<&ItemRecord> = items.iter().filter(|item| item.is_active).collect();
    let item_ids: BTreeSet<String> = active_items
        .iter()
        .map(|item| item.item_id.clone())
        .collect();

    let mut customer_history: HashMap<String, Vec<&OrderItemHistoryRow>> = HashMap::new();
    for row in history {
        customer_history
            .entry(row.customer_id.clone())
            .or_default()
            .push(row);
    }
    for rows in customer_history.values_mut() {
        rows.sort_by(|a, b| {
            a.ordered_at
                .cmp(&b.ordered_at)
                .then(a.order_id.cmp(&b.order_id))
        });
    }

    let transition_map = build_transition_probabilities(&customer_history);
    let segment_popularity = build_segment_popularity(customers, history);

    let mut all_scores = Vec::new();

    for customer in customers {
        let history_rows = customer_history.get(&customer.customer_id);
        let candidates = candidate_items(
            history_rows,
            &transition_map,
            &segment_popularity,
            customer,
            &item_ids,
        );
        if candidates.is_empty() {
            continue;
        }

        let repeat_scores = build_repeat_scores(history_rows);
        let transition_scores = build_transition_scores(history_rows, &transition_map);
        let segment_scores = build_segment_scores(customer, &segment_popularity);

        let mut ranked = candidates
            .into_iter()
            .map(|item_id| {
                let score = weights.repeat * repeat_scores.get(&item_id).copied().unwrap_or(0.0)
                    + weights.transition * transition_scores.get(&item_id).copied().unwrap_or(0.0)
                    + weights.segment * segment_scores.get(&item_id).copied().unwrap_or(0.0);
                (item_id, score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect::<Vec<_>>();

        ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        ranked.truncate(top_n);

        all_scores.extend(
            ranked
                .into_iter()
                .enumerate()
                .map(|(idx, (item_id, score))| CandidateScore {
                    customer_id: customer.customer_id.clone(),
                    item_id,
                    score,
                    rank: (idx + 1) as i32,
                }),
        );
    }

    let forecasts = build_forecasts(&all_scores);
    (all_scores, forecasts)
}

fn build_repeat_scores(history_rows: Option<&Vec<&OrderItemHistoryRow>>) -> HashMap<String, f64> {
    let mut counts: HashMap<String, f64> = HashMap::new();
    let Some(rows) = history_rows else {
        return counts;
    };

    let now = Utc::now().date_naive();
    for row in rows {
        let recency = row
            .ordered_at
            .map(|ordered_at| {
                let days = (now - ordered_at.date_naive()).num_days().max(0) as f64;
                1.0 / (1.0 + days / 30.0)
            })
            .unwrap_or(0.5);
        *counts.entry(row.item_id.clone()).or_insert(0.0) += row.quantity as f64 * recency;
    }

    normalize_map(counts)
}

fn build_transition_probabilities(
    customer_history: &HashMap<String, Vec<&OrderItemHistoryRow>>,
) -> HashMap<String, HashMap<String, f64>> {
    let mut transitions: HashMap<String, HashMap<String, f64>> = HashMap::new();

    for rows in customer_history.values() {
        for window in rows.windows(2) {
            let prev = window[0].item_id.clone();
            let next = window[1].item_id.clone();
            *transitions
                .entry(prev)
                .or_default()
                .entry(next)
                .or_insert(0.0) += 1.0;
        }
    }

    transitions
        .into_iter()
        .map(|(from_item, counts)| (from_item, normalize_map(counts)))
        .collect()
}

fn build_transition_scores(
    history_rows: Option<&Vec<&OrderItemHistoryRow>>,
    transitions: &HashMap<String, HashMap<String, f64>>,
) -> HashMap<String, f64> {
    let Some(last_item) = history_rows
        .and_then(|rows| rows.last())
        .map(|row| row.item_id.clone())
    else {
        return HashMap::new();
    };

    transitions.get(&last_item).cloned().unwrap_or_default()
}

fn build_segment_popularity(
    customers: &[CustomerRecord],
    history: &[OrderItemHistoryRow],
) -> HashMap<(String, String, String), HashMap<String, f64>> {
    let customer_segment: HashMap<String, (String, String, String)> = customers
        .iter()
        .map(|customer| {
            (
                customer.customer_id.clone(),
                (
                    customer.country.clone().unwrap_or_default(),
                    customer.status.clone().unwrap_or_default(),
                    customer.tier.clone().unwrap_or_default(),
                ),
            )
        })
        .collect();

    let mut segment_counts: HashMap<(String, String, String), HashMap<String, f64>> =
        HashMap::new();
    for row in history {
        if let Some(segment) = customer_segment.get(&row.customer_id) {
            *segment_counts
                .entry(segment.clone())
                .or_default()
                .entry(row.item_id.clone())
                .or_insert(0.0) += row.quantity as f64;
        }
    }

    segment_counts
        .into_iter()
        .map(|(segment, counts)| (segment, normalize_map(counts)))
        .collect()
}

fn build_segment_scores(
    customer: &CustomerRecord,
    segment_popularity: &HashMap<(String, String, String), HashMap<String, f64>>,
) -> HashMap<String, f64> {
    let key = (
        customer.country.clone().unwrap_or_default(),
        customer.status.clone().unwrap_or_default(),
        customer.tier.clone().unwrap_or_default(),
    );
    segment_popularity.get(&key).cloned().unwrap_or_default()
}

fn candidate_items(
    history_rows: Option<&Vec<&OrderItemHistoryRow>>,
    transitions: &HashMap<String, HashMap<String, f64>>,
    segment_popularity: &HashMap<(String, String, String), HashMap<String, f64>>,
    customer: &CustomerRecord,
    all_item_ids: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut candidates = BTreeSet::new();

    if let Some(rows) = history_rows {
        for row in rows {
            candidates.insert(row.item_id.clone());
        }
        if let Some(last_item) = rows.last().map(|row| row.item_id.clone()) {
            if let Some(next_map) = transitions.get(&last_item) {
                candidates.extend(next_map.keys().cloned());
            }
        }
    }

    let segment_key = (
        customer.country.clone().unwrap_or_default(),
        customer.status.clone().unwrap_or_default(),
        customer.tier.clone().unwrap_or_default(),
    );
    if let Some(popular) = segment_popularity.get(&segment_key) {
        candidates.extend(popular.keys().cloned());
    }

    if candidates.is_empty() {
        candidates.extend(all_item_ids.iter().take(20).cloned());
    }

    candidates
}

fn build_forecasts(scores: &[CandidateScore]) -> Vec<DemandForecastRow> {
    let mut sums: BTreeMap<String, f64> = BTreeMap::new();
    for row in scores {
        *sums.entry(row.item_id.clone()).or_insert(0.0) += row.score;
    }

    sums.into_iter()
        .map(|(item_id, total_score)| {
            let expected = if total_score <= 0.0 {
                0
            } else {
                total_score.round().max(1.0) as i32
            };
            let low = ((expected as f64) * 0.8).floor() as i32;
            let high = ((expected as f64) * 1.2).ceil() as i32;
            DemandForecastRow {
                item_id,
                expected_qty: expected,
                low_qty: low.max(0),
                high_qty: high.max(expected),
            }
        })
        .collect()
}

fn normalize_map(mut values: HashMap<String, f64>) -> HashMap<String, f64> {
    let max = values.values().copied().fold(0.0, f64::max);
    if max <= 0.0 {
        return HashMap::new();
    }
    for value in values.values_mut() {
        *value /= max;
    }
    values
}

#[cfg(test)]
mod tests {
    use super::{Weights, build_scores};
    use crate::db::{CustomerRecord, ItemRecord, OrderItemHistoryRow};
    use chrono::{Duration, Utc};

    #[test]
    fn builds_ranked_scores_and_forecasts() {
        let now = Utc::now();
        let customers = vec![CustomerRecord {
            customer_id: "C1".into(),
            country: Some("Japan".into()),
            status: Some("active".into()),
            tier: Some("gold".into()),
        }];
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
        ];
        let history = vec![
            OrderItemHistoryRow {
                customer_id: "C1".into(),
                order_id: "O1".into(),
                ordered_at: Some(now - Duration::days(10)),
                item_id: "A".into(),
                quantity: 2,
            },
            OrderItemHistoryRow {
                customer_id: "C1".into(),
                order_id: "O2".into(),
                ordered_at: Some(now - Duration::days(1)),
                item_id: "B".into(),
                quantity: 1,
            },
        ];

        let (scores, forecasts) = build_scores(
            &customers,
            &items,
            &history,
            5,
            Weights {
                repeat: 0.5,
                transition: 0.3,
                segment: 0.2,
            },
        );

        assert!(!scores.is_empty());
        assert!(!forecasts.is_empty());
    }
}
