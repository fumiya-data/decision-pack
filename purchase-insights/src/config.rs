use std::env;

#[derive(Debug, Clone)]
pub struct CliConfig {
    pub database_url: String,
    pub run_id: String,
    pub top_n: usize,
    pub weight_repeat: f64,
    pub weight_transition: f64,
    pub weight_segment: f64,
}

impl CliConfig {
    pub fn parse_from_env() -> Result<Self, String> {
        let mut database_url = None;
        let mut run_id = None;
        let mut top_n = None;
        let mut weight_repeat = None;
        let mut weight_transition = None;
        let mut weight_segment = None;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--database-url" => database_url = Some(next_string(&mut args, "--database-url")?),
                "--run-id" => run_id = Some(next_string(&mut args, "--run-id")?),
                "--top-n" => {
                    top_n = Some(
                        next_string(&mut args, "--top-n")?
                            .parse()
                            .map_err(|_| "--top-n は数値で指定してください".to_string())?,
                    )
                }
                "--weight-repeat" => weight_repeat = Some(parse_f64(&mut args, "--weight-repeat")?),
                "--weight-transition" => {
                    weight_transition = Some(parse_f64(&mut args, "--weight-transition")?)
                }
                "--weight-segment" => {
                    weight_segment = Some(parse_f64(&mut args, "--weight-segment")?)
                }
                "--help" | "-h" => return Err(Self::usage()),
                other => return Err(format!("未知の引数です: `{other}`\n\n{}", Self::usage())),
            }
        }

        let database_url = database_url
            .or_else(|| env::var("DATABASE_URL").ok())
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| format!("--database-url は必須です\n\n{}", Self::usage()))?;

        Ok(Self {
            database_url,
            run_id: run_id.unwrap_or_else(|| "local".to_string()),
            top_n: top_n.unwrap_or(5),
            weight_repeat: weight_repeat.unwrap_or(0.5),
            weight_transition: weight_transition.unwrap_or(0.3),
            weight_segment: weight_segment.unwrap_or(0.2),
        })
    }

    fn usage() -> String {
        "使い方: cargo run -p purchase-insights -- --database-url <URL> [--run-id <ID>] [--top-n <N>] [--weight-repeat <F>] [--weight-transition <F>] [--weight-segment <F>]".to_string()
    }
}

fn next_string(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{flag} に値がありません"))
}

fn parse_f64(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<f64, String> {
    next_string(args, flag)?
        .parse()
        .map_err(|_| format!("{flag} は数値で指定してください"))
}
