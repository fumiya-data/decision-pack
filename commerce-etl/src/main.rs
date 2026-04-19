use commerce_etl::config::CliConfig;
use commerce_etl::csv_input::{InventoryRow, ItemRow, OrderItemRow, OrderRow, read_csv};
use commerce_etl::persistence::persist_all;
use commerce_etl::sample::generate_sample_files;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = CliConfig::parse_from_env()
        .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message))?;

    match config {
        CliConfig::Import(config) => {
            let items = maybe_read::<ItemRow>(config.items_csv.as_deref())?;
            let orders = maybe_read::<OrderRow>(config.orders_csv.as_deref())?;
            let order_items = maybe_read::<OrderItemRow>(config.order_items_csv.as_deref())?;
            let inventory = maybe_read::<InventoryRow>(config.inventory_csv.as_deref())?;

            let summary = persist_all(&config, &items, &orders, &order_items, &inventory).await?;

            println!("commerce-etl の取込が完了しました");
            println!("  items: {}", summary.items_upserted);
            println!("  orders: {}", summary.orders_upserted);
            println!("  order_items: {}", summary.order_items_upserted);
            println!("  inventory_balance: {}", summary.inventory_upserted);
        }
        CliConfig::GenerateSample(config) => {
            let summary = generate_sample_files(&config)?;
            println!("commerce-etl の評価用サンプル生成が完了しました");
            println!("  customers: {}", summary.customers);
            println!("  items: {}", summary.items);
            println!("  orders: {}", summary.orders);
            println!("  order_items: {}", summary.order_items);
            println!("  repeat_customers: {}", summary.repeat_customers);
        }
    }

    Ok(())
}

fn maybe_read<T>(path: Option<&std::path::Path>) -> Result<Vec<T>, Box<dyn std::error::Error>>
where
    T: for<'de> serde::Deserialize<'de>,
{
    match path {
        Some(path) => read_csv(path),
        None => Ok(Vec::new()),
    }
}
