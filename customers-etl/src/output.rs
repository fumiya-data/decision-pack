//! ETL CLI のファイル出力補助です。

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use csv::Writer;
use serde::Serialize;

use crate::config::CliConfig;
use crate::formatter::FormatRun;
use crate::report::IssueKind;
use crate::schema::Column;
use crate::summary::SegmentSummary;

pub const FORMATTED_CSV_NAME: &str = "formatted.csv";
pub const ISSUE_LOG_CSV_NAME: &str = "format_issues.csv";
pub const SEGMENT_SUMMARY_CSV_NAME: &str = "customer_segment_summary.csv";
pub const RUN_SUMMARY_JSON_NAME: &str = "run_summary.json";

/// 1 回の CLI 実行で出力されるファイル群のパスです。
#[derive(Debug, Clone)]
pub struct OutputPaths {
    pub formatted_csv: PathBuf,
    pub issue_log_csv: PathBuf,
    pub segment_summary_csv: PathBuf,
    pub run_summary_json: PathBuf,
}

impl OutputPaths {
    pub fn from_dir(out_dir: &Path) -> Self {
        Self {
            formatted_csv: out_dir.join(FORMATTED_CSV_NAME),
            issue_log_csv: out_dir.join(ISSUE_LOG_CSV_NAME),
            segment_summary_csv: out_dir.join(SEGMENT_SUMMARY_CSV_NAME),
            run_summary_json: out_dir.join(RUN_SUMMARY_JSON_NAME),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RunSummary {
    pub job_name: &'static str,
    pub run_id: String,
    pub status: &'static str,
    pub started_at: String,
    pub finished_at: String,
    pub input_path: String,
    pub output_dir: String,
    pub counts: RunSummaryCounts,
    pub artifacts: ArtifactNames,
}

#[derive(Debug, Serialize)]
pub struct RunSummaryCounts {
    pub data_rows_seen: usize,
    pub rows_written: usize,
    pub rows_with_failures: usize,
    pub skipped_rows: usize,
    pub malformed_rows: usize,
    pub segment_rows_written: usize,
    pub segment_rows_excluded_invalid_keys: usize,
}

#[derive(Debug, Serialize)]
pub struct ArtifactNames {
    pub formatted_csv: &'static str,
    pub format_issues_csv: &'static str,
    pub customer_segment_summary_csv: &'static str,
}

pub fn ensure_output_dir(path: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(path)?;
    Ok(())
}

/// 整形済みデータセットを書き出します。
pub fn write_cleaned_output(run: &FormatRun, out_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(out_path)?;
    for row in &run.cleaned_rows {
        writer.write_record(row)?;
    }
    writer.flush()?;
    Ok(())
}

/// 失敗やスキップを確認するための構造化 issue log を書き出します。
pub fn write_issue_log(run: &FormatRun, out_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(out_path)?;
    writer.write_record([
        "line_number",
        "kind",
        "column",
        "raw_value",
        "output_value",
        "reason",
    ])?;

    for issue in &run.report.issues {
        writer.write_record([
            issue.line_number.to_string(),
            issue.kind.as_str().to_string(),
            issue
                .column
                .map(|column| column.header().to_string())
                .unwrap_or_else(|| "__row__".to_string()),
            issue.raw_value.clone(),
            issue.output_value.clone(),
            issue.reason.clone(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

/// 下流利用向けの安定した顧客セグメント要約を書き出します。
pub fn write_segment_summary(
    summary: &SegmentSummary,
    out_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(out_path)?;
    writer.write_record([
        "country",
        "status",
        "tier",
        "customer_count",
        "marketing_opt_in_true_count",
        "total_spend_known_count",
        "total_spend_sum",
        "total_spend_avg",
        "order_count_known_count",
        "order_count_sum",
        "order_count_avg",
        "last_purchase_known_count",
    ])?;

    for row in &summary.rows {
        writer.write_record([
            row.country.clone(),
            row.status.clone(),
            row.tier.clone(),
            row.customer_count.to_string(),
            row.marketing_opt_in_true_count.to_string(),
            row.total_spend_known_count.to_string(),
            format!("{:.2}", row.total_spend_sum),
            format!("{:.2}", row.total_spend_avg),
            row.order_count_known_count.to_string(),
            row.order_count_sum.to_string(),
            format!("{:.2}", row.order_count_avg),
            row.last_purchase_known_count.to_string(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

/// 運用追跡用の JSON 実行要約を書き出します。
pub fn write_run_summary(
    config: &CliConfig,
    run: &FormatRun,
    summary: &SegmentSummary,
    started_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    out_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let summary_doc = RunSummary {
        job_name: "customers-etl",
        run_id: config.run_id.clone(),
        status: "succeeded",
        started_at: started_at.to_rfc3339(),
        finished_at: finished_at.to_rfc3339(),
        input_path: config.input.display().to_string(),
        output_dir: config.output_dir.display().to_string(),
        counts: RunSummaryCounts {
            data_rows_seen: run.report.data_rows_seen,
            rows_written: run.report.rows_written,
            rows_with_failures: run.report.rows_with_failures,
            skipped_rows: run.report.skipped_rows,
            malformed_rows: run.report.malformed_rows,
            segment_rows_written: summary.rows.len(),
            segment_rows_excluded_invalid_keys: summary.excluded_invalid_keys,
        },
        artifacts: ArtifactNames {
            formatted_csv: FORMATTED_CSV_NAME,
            format_issues_csv: ISSUE_LOG_CSV_NAME,
            customer_segment_summary_csv: SEGMENT_SUMMARY_CSV_NAME,
        },
    };

    fs::write(out_path, serde_json::to_string_pretty(&summary_doc)?)?;
    Ok(())
}

/// どこまで整形できたか、下流向け集計を何行出力したかを人が素早く確認できるように表示します。
pub fn print_summary(run: &FormatRun, summary: &SegmentSummary, paths: &OutputPaths) {
    println!("整形処理が完了しました");
    println!("  整形済み出力: {}", paths.formatted_csv.display());
    println!("  issue log: {}", paths.issue_log_csv.display());
    println!("  セグメント要約: {}", paths.segment_summary_csv.display());
    println!("  実行要約: {}", paths.run_summary_json.display());

    println!("\n行集計");
    println!("  解析対象データ行: {}", run.report.data_rows_seen);
    println!("  出力行数: {}", run.report.rows_written);
    println!(
        "  フィールド失敗を含む行数: {}",
        run.report.rows_with_failures
    );
    println!("  スキップ行数: {}", run.report.skipped_rows);
    println!("  不正行数: {}", run.report.malformed_rows);
    println!("  セグメント要約行数: {}", summary.rows.len());
    println!(
        "  無効キーにより除外したセグメント行数: {}",
        summary.excluded_invalid_keys
    );

    println!("\n列ごとの結果");
    for column in Column::ALL {
        let stats = run.report.column_stats(column);
        println!(
            "  {:<20} 成功 {:>4}  空値 {:>4}  失敗 {:>4}",
            column.header(),
            stats.success,
            stats.empty,
            stats.failed
        );
    }

    let sample_issues: Vec<_> = run.report.issues.iter().take(15).collect();
    if !sample_issues.is_empty() {
        println!("\n失敗・スキップのサンプル");
        for issue in sample_issues {
            let scope = match issue.kind {
                IssueKind::FieldFailed => issue
                    .column
                    .map(|column| column.header())
                    .unwrap_or("__row__"),
                _ => "__row__",
            };
            println!(
                "  行 {} [{}] `{}` ({})",
                issue.line_number, scope, issue.raw_value, issue.reason
            );
        }
    }
}
