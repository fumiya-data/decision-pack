//! customers ETL バイナリ用の CLI 設定を解釈します。

use std::env;
use std::path::PathBuf;

/// 1 回の ETL 実行に対応するコマンドライン設定です。
#[derive(Debug, Clone)]
pub struct CliConfig {
    pub input: PathBuf,
    pub output_dir: PathBuf,
    pub run_id: String,
}

impl CliConfig {
    /// プロセス引数から `--input`、`--output-dir`、任意の `--run-id` を読み取ります。
    pub fn parse_from_env() -> Result<Self, String> {
        let mut input = None;
        let mut output_dir = None;
        let mut run_id = None;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
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
                "--help" | "-h" => return Err(Self::usage()),
                other => {
                    return Err(format!("未知の引数です: `{other}`\n\n{}", Self::usage()));
                }
            }
        }

        let input = input.ok_or_else(|| format!("--input は必須です\n\n{}", Self::usage()))?;
        let output_dir =
            output_dir.ok_or_else(|| format!("--output-dir は必須です\n\n{}", Self::usage()))?;

        Ok(Self {
            input,
            output_dir,
            run_id: run_id.unwrap_or_else(|| "local".to_string()),
        })
    }

    fn usage() -> String {
        "使い方: cargo run -p customers-etl -- --input <PATH> --output-dir <DIR> [--run-id <ID>]"
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::CliConfig;

    #[test]
    fn usage_mentions_required_flags() {
        let usage = CliConfig::usage();
        assert!(usage.contains("--input"));
        assert!(usage.contains("--output-dir"));
    }
}
