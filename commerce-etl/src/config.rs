use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CliConfig {
    pub items_csv: Option<PathBuf>,
    pub orders_csv: Option<PathBuf>,
    pub order_items_csv: Option<PathBuf>,
    pub inventory_csv: Option<PathBuf>,
    pub database_url: String,
    pub run_id: String,
}

impl CliConfig {
    pub fn parse_from_env() -> Result<Self, String> {
        let mut items_csv = None;
        let mut orders_csv = None;
        let mut order_items_csv = None;
        let mut inventory_csv = None;
        let mut database_url = None;
        let mut run_id = None;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--items-csv" => items_csv = Some(next_value(&mut args, "--items-csv")?),
                "--orders-csv" => orders_csv = Some(next_value(&mut args, "--orders-csv")?),
                "--order-items-csv" => {
                    order_items_csv = Some(next_value(&mut args, "--order-items-csv")?)
                }
                "--inventory-csv" => {
                    inventory_csv = Some(next_value(&mut args, "--inventory-csv")?)
                }
                "--database-url" => database_url = Some(next_string(&mut args, "--database-url")?),
                "--run-id" => run_id = Some(next_string(&mut args, "--run-id")?),
                "--help" | "-h" => return Err(Self::usage()),
                other => return Err(format!("未知の引数です: `{other}`\n\n{}", Self::usage())),
            }
        }

        if items_csv.is_none()
            && orders_csv.is_none()
            && order_items_csv.is_none()
            && inventory_csv.is_none()
        {
            return Err(format!(
                "少なくとも 1 つの入力 CSV を指定してください\n\n{}",
                Self::usage()
            ));
        }

        let database_url = database_url
            .or_else(|| env::var("DATABASE_URL").ok())
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| format!("--database-url は必須です\n\n{}", Self::usage()))?;

        Ok(Self {
            items_csv,
            orders_csv,
            order_items_csv,
            inventory_csv,
            database_url,
            run_id: run_id.unwrap_or_else(|| "local".to_string()),
        })
    }

    fn usage() -> String {
        "使い方: cargo run -p commerce-etl -- [--items-csv <PATH>] [--orders-csv <PATH>] [--order-items-csv <PATH>] [--inventory-csv <PATH>] --database-url <URL> [--run-id <ID>]".to_string()
    }
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(next_string(args, flag)?))
}

fn next_string(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{flag} に値がありません"))
}
