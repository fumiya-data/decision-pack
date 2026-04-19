//! customers ETL バイナリ用の CLI 設定を解釈します。

use std::env;
use std::path::PathBuf;

/// CLI が受け付けるコマンド種別です。
#[derive(Debug, Clone)]
pub enum CliCommand {
    Format(FormatConfig),
    GenerateRawSample(GenerateRawSampleConfig),
}

/// 1 回の ETL 実行に対応するコマンドライン設定です。
#[derive(Debug, Clone)]
pub struct FormatConfig {
    pub input: PathBuf,
    pub output_dir: PathBuf,
    pub run_id: String,
    pub database_url: Option<String>,
}

/// 未整形顧客 raw サンプル生成に対応するコマンドライン設定です。
#[derive(Debug, Clone)]
pub struct GenerateRawSampleConfig {
    pub output_raw: PathBuf,
    pub target_formatted_count: usize,
    pub invalid_row_count: usize,
    pub seed: u64,
}

impl CliCommand {
    /// プロセス引数から通常 ETL 実行か raw サンプル生成かを読み取ります。
    pub fn parse_from_env() -> Result<Self, String> {
        let mut input = None;
        let mut output_dir = None;
        let mut run_id = None;
        let mut database_url = None;

        let mut output_raw = None;
        let mut target_formatted_count = None;
        let mut invalid_row_count = None;
        let mut seed = None;
        let mut generate_raw_sample = false;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--generate-raw-sample" => {
                    generate_raw_sample = true;
                }
                "--input" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--input に値がありません".to_string())?;
                    input = Some(PathBuf::from(value));
                }
                "--output-dir" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--output-dir に値がありません".to_string())?;
                    output_dir = Some(PathBuf::from(value));
                }
                "--run-id" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--run-id に値がありません".to_string())?;
                    run_id = Some(value);
                }
                "--database-url" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--database-url に値がありません".to_string())?;
                    database_url = Some(value);
                }
                "--output-raw" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--output-raw に値がありません".to_string())?;
                    output_raw = Some(PathBuf::from(value));
                }
                "--target-formatted-count" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--target-formatted-count に値がありません".to_string())?;
                    target_formatted_count = Some(value.parse::<usize>().map_err(|_| {
                        "--target-formatted-count は正の整数で指定してください".to_string()
                    })?);
                }
                "--invalid-row-count" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--invalid-row-count に値がありません".to_string())?;
                    invalid_row_count = Some(value.parse::<usize>().map_err(|_| {
                        "--invalid-row-count は 0 以上の整数で指定してください".to_string()
                    })?);
                }
                "--seed" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--seed に値がありません".to_string())?;
                    seed = Some(
                        value
                            .parse::<u64>()
                            .map_err(|_| "--seed は 0 以上の整数で指定してください".to_string())?,
                    );
                }
                "--help" | "-h" => return Err(Self::usage()),
                other => {
                    return Err(format!("未知の引数です: `{other}`\n\n{}", Self::usage()));
                }
            }
        }

        if generate_raw_sample {
            let output_raw = output_raw
                .ok_or_else(|| format!("--output-raw は必須です\n\n{}", Self::usage()))?;
            return Ok(Self::GenerateRawSample(GenerateRawSampleConfig {
                output_raw,
                target_formatted_count: target_formatted_count.unwrap_or(50_000),
                invalid_row_count: invalid_row_count.unwrap_or(240),
                seed: seed.unwrap_or(20_260_419),
            }));
        }

        let input = input.ok_or_else(|| format!("--input は必須です\n\n{}", Self::usage()))?;
        let output_dir =
            output_dir.ok_or_else(|| format!("--output-dir は必須です\n\n{}", Self::usage()))?;

        Ok(Self::Format(FormatConfig {
            input,
            output_dir,
            run_id: run_id.unwrap_or_else(|| "local".to_string()),
            database_url: database_url
                .or_else(|| env::var("DATABASE_URL").ok())
                .filter(|value| !value.trim().is_empty()),
        }))
    }

    fn usage() -> String {
        [
            "使い方:",
            "  整形実行: cargo run -p customers-etl -- --input <PATH> --output-dir <DIR> [--run-id <ID>] [--database-url <URL>]",
            "  raw 生成: cargo run -p customers-etl -- --generate-raw-sample --output-raw <PATH> [--target-formatted-count <N>] [--invalid-row-count <N>] [--seed <N>]",
        ]
        .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::CliCommand;

    #[test]
    fn usage_mentions_required_flags() {
        let usage = CliCommand::usage();
        assert!(usage.contains("--input"));
        assert!(usage.contains("--output-dir"));
        assert!(usage.contains("--generate-raw-sample"));
    }
}
