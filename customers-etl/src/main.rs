//! 全件整形バイナリのエントリポイントです。
//!
//! `cargo run` を実行すると次を順に行います。
//! 1. `--input` で指定した CSV を読む
//! 2. 復旧可能な行と列をすべて整形する
//! 3. `formatted.csv` を書き出す
//! 4. `format_issues.csv` を書き出す
//! 5. `customer_segment_summary.csv` を書き出す
//! 6. `run_summary.json` を書き出す
//! 7. 成功件数、空値件数、失敗件数、スキップ件数、不正行件数、
//!    セグメント集計行数を人間向けに表示する

use std::error::Error;
use std::fs;

use chrono::Utc;
use customers_etl::config::{CliCommand, FormatConfig};
use customers_etl::output::{
    OutputPaths, ensure_output_dir, print_summary, write_cleaned_output, write_issue_log,
    write_run_summary, write_segment_summary,
};
use customers_etl::persistence::persist_run;
use customers_etl::{build_segment_summary, format_dataset, generate_raw_sample};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let command = CliCommand::parse_from_env()
        .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message))?;

    match command {
        CliCommand::Format(config) => run_format(config).await,
        CliCommand::GenerateRawSample(config) => {
            let summary = generate_raw_sample(&config)?;
            println!(
                "未整形 raw サンプルを生成しました\n  output_raw: {}\n  output_metadata: {}\n  target_formatted_count: {}\n  raw_rows_written: {}\n  embedded_headers: {}\n  invalid_rows: {}",
                summary.output_raw,
                summary.output_metadata,
                summary.target_formatted_count,
                summary.raw_rows_written,
                summary.embedded_headers_written,
                summary.invalid_rows_written
            );
            println!("  countries: {:?}", summary.country_counts);
            println!("  languages: {:?}", summary.language_counts);
            Ok(())
        }
    }
}

async fn run_format(config: FormatConfig) -> Result<(), Box<dyn Error>> {
    let started_at = Utc::now();
    let input = fs::read_to_string(&config.input)?;
    let run = format_dataset(&input);
    let segment_summary = build_segment_summary(&run.cleaned_rows);
    let output_paths = OutputPaths::from_dir(&config.output_dir);

    ensure_output_dir(&config.output_dir)?;
    write_cleaned_output(&run, &output_paths.formatted_csv)?;
    write_issue_log(&run, &output_paths.issue_log_csv)?;
    write_segment_summary(&segment_summary, &output_paths.segment_summary_csv)?;
    write_run_summary(
        &config,
        &run,
        &segment_summary,
        started_at,
        Utc::now(),
        &output_paths.run_summary_json,
    )?;
    print_summary(&run, &segment_summary, &output_paths);

    if config.database_url.is_some() {
        let persist_summary = persist_run(&config, &run).await?;
        println!(
            "\nPostgreSQL への保存が完了しました\n  customers: {}\n  customer_load_issues: {}\n  persist_skipped_rows: {}\n  etl_job_runs: 1",
            persist_summary.customers_upserted,
            persist_summary.issues_inserted,
            persist_summary.rows_skipped_for_persist
        );
    }

    Ok(())
}
