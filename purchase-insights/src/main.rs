use purchase_insights::algorithm::{Weights, build_scores};
use purchase_insights::config::CliConfig;
use purchase_insights::db::{load_customers, load_items, load_order_history, persist_results};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = CliConfig::parse_from_env()
        .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message))?;

    let customers = load_customers(&config.database_url).await?;
    let items = load_items(&config.database_url).await?;
    let history = load_order_history(&config.database_url).await?;

    let (scores, forecasts) = build_scores(
        &customers,
        &items,
        &history,
        config.top_n,
        Weights {
            repeat: config.weight_repeat,
            transition: config.weight_transition,
            segment: config.weight_segment,
        },
    );

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
