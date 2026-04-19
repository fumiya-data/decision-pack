use purchase_insights::algorithm::{Weights, build_scores};
use purchase_insights::config::CliConfig;
use purchase_insights::db::{load_customers, load_items, load_order_history, persist_results};
use purchase_insights::evaluation::evaluate_recommender;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = CliConfig::parse_from_env()
        .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message))?;

    let customers = load_customers(&config.database_url).await?;
    let items = load_items(&config.database_url).await?;
    let history = load_order_history(&config.database_url).await?;
    let weights = Weights {
        repeat: config.weight_repeat,
        transition: config.weight_transition,
        segment: config.weight_segment,
    };

    if config.evaluate {
        let report = evaluate_recommender(
            &customers,
            &items,
            &history,
            config.top_n,
            weights,
            config.min_orders_for_eval,
        );
        println!("purchase-insights のオフライン評価");
        println!("  eligible_customers: {}", report.eligible_customers);
        println!(
            "  customers_with_predictions: {}",
            report.customers_with_predictions
        );
        println!(
            "  model: hit@3={:.4} hit@5={:.4} recall@5={:.4} recall@10={:.4} ndcg@5={:.4} ndcg@10={:.4}",
            report.model.hit_at_3,
            report.model.hit_at_5,
            report.model.recall_at_5,
            report.model.recall_at_10,
            report.model.ndcg_at_5,
            report.model.ndcg_at_10
        );
        println!(
            "  popularity_baseline: hit@3={:.4} hit@5={:.4} recall@5={:.4} recall@10={:.4} ndcg@5={:.4} ndcg@10={:.4}",
            report.popularity_baseline.hit_at_3,
            report.popularity_baseline.hit_at_5,
            report.popularity_baseline.recall_at_5,
            report.popularity_baseline.recall_at_10,
            report.popularity_baseline.ndcg_at_5,
            report.popularity_baseline.ndcg_at_10
        );
    }

    let (scores, forecasts) = build_scores(&customers, &items, &history, config.top_n, weights);

    if config.skip_persist {
        println!("purchase-insights の集約を計算しましたが、--skip-persist により保存は行いません");
        println!("  customer_item_next_buy_score: {}", scores.len());
        println!("  item_demand_forecast: {}", forecasts.len());
        return Ok(());
    }

    let summary =
        persist_results(&config.database_url, &config.run_id, &scores, &forecasts).await?;

    println!("purchase-insights の集約が完了しました");
    println!(
        "  customer_item_next_buy_score: {}",
        summary.scores_inserted
    );
    println!("  item_demand_forecast: {}", summary.forecasts_inserted);

    Ok(())
}
