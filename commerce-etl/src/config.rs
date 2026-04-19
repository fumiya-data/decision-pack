use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ImportConfig {
    pub items_csv: Option<PathBuf>,
    pub orders_csv: Option<PathBuf>,
    pub order_items_csv: Option<PathBuf>,
    pub inventory_csv: Option<PathBuf>,
    pub database_url: String,
    pub run_id: String,
}

#[derive(Debug, Clone)]
pub struct SampleConfig {
    pub customers_csv: PathBuf,
    pub output_dir: PathBuf,
    pub customer_count: usize,
    pub item_count: usize,
    pub order_count: usize,
    pub seed: u64,
}

#[derive(Debug, Clone)]
pub enum CliConfig {
    Import(ImportConfig),
    GenerateSample(SampleConfig),
}

impl CliConfig {
    pub fn parse_from_env() -> Result<Self, String> {
        let mut items_csv = None;
        let mut orders_csv = None;
        let mut order_items_csv = None;
        let mut inventory_csv = None;
        let mut database_url = None;
        let mut run_id = None;
        let mut generate_sample = false;
        let mut customers_csv = None;
        let mut output_dir = None;
        let mut customer_count = None;
        let mut item_count = None;
        let mut order_count = None;
        let mut seed = None;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--generate-sample" => generate_sample = true,
                "--items-csv" => items_csv = Some(next_value(&mut args, "--items-csv")?),
                "--orders-csv" => orders_csv = Some(next_value(&mut args, "--orders-csv")?),
                "--order-items-csv" => {
                    order_items_csv = Some(next_value(&mut args, "--order-items-csv")?)
                }
                "--inventory-csv" => {
                    inventory_csv = Some(next_value(&mut args, "--inventory-csv")?)
                }
                "--customers-csv" => {
                    customers_csv = Some(next_value(&mut args, "--customers-csv")?)
                }
                "--output-dir" => output_dir = Some(next_value(&mut args, "--output-dir")?),
                "--database-url" => database_url = Some(next_string(&mut args, "--database-url")?),
                "--run-id" => run_id = Some(next_string(&mut args, "--run-id")?),
                "--customer-count" => {
                    customer_count = Some(parse_usize(&mut args, "--customer-count")?)
                }
                "--item-count" => item_count = Some(parse_usize(&mut args, "--item-count")?),
                "--order-count" => order_count = Some(parse_usize(&mut args, "--order-count")?),
                "--seed" => seed = Some(parse_u64(&mut args, "--seed")?),
                "--help" | "-h" => return Err(Self::usage()),
                other => return Err(format!("未知の引数です: `{other}`\n\n{}", Self::usage())),
            }
        }

        if generate_sample {
            let customers_csv = customers_csv.ok_or_else(|| {
                format!(
                    "--generate-sample では --customers-csv が必須です\n\n{}",
                    Self::usage()
                )
            })?;
            let output_dir = output_dir.ok_or_else(|| {
                format!(
                    "--generate-sample では --output-dir が必須です\n\n{}",
                    Self::usage()
                )
            })?;

            return Ok(Self::GenerateSample(SampleConfig {
                customers_csv,
                output_dir,
                customer_count: customer_count.unwrap_or(50_000),
                item_count: item_count.unwrap_or(100),
                order_count: order_count.unwrap_or(150_000),
                seed: seed.unwrap_or(2026_0417),
            }));
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

        Ok(Self::Import(ImportConfig {
            items_csv,
            orders_csv,
            order_items_csv,
            inventory_csv,
            database_url,
            run_id: run_id.unwrap_or_else(|| "local".to_string()),
        }))
    }

    fn usage() -> String {
        "使い方:\n  取込: cargo run -p commerce-etl -- [--items-csv <PATH>] [--orders-csv <PATH>] [--order-items-csv <PATH>] [--inventory-csv <PATH>] --database-url <URL> [--run-id <ID>]\n  サンプル生成: cargo run -p commerce-etl -- --generate-sample --customers-csv <PATH> --output-dir <DIR> [--customer-count <N>] [--item-count <N>] [--order-count <N>] [--seed <N>]".to_string()
    }
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(next_string(args, flag)?))
}

fn next_string(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{flag} に値がありません"))
}

fn parse_usize(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<usize, String> {
    next_string(args, flag)?
        .parse()
        .map_err(|_| format!("{flag} は整数で指定してください"))
}

fn parse_u64(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<u64, String> {
    next_string(args, flag)?
        .parse()
        .map_err(|_| format!("{flag} は整数で指定してください"))
}
