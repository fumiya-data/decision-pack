//! 行復旧、列整形、レポート収集を end-to-end で統括します。

use crate::columns;
use crate::csv_input::{InputRecord, parse_input};
use crate::report::RunReport;
use crate::schema::{Column, HEADER_ROW};

/// 1 つの入力ファイルに対して整形を実行した最終結果です。
#[derive(Debug, Clone)]
pub struct FormatRun {
    pub cleaned_rows: Vec<Vec<String>>,
    pub report: RunReport,
}

/// 入力データセット中の復旧可能な行をすべて整形します。
///
/// 常に正規ヘッダ行を書き出し、非データ行はスキップし、不正行は issue log にだけ残し、
/// 各値の成功・空値化・失敗判定は列処理側へ委ねます。
pub fn format_dataset(input: &str) -> FormatRun {
    let mut cleaned_rows = vec![HEADER_ROW.iter().map(|value| value.to_string()).collect()];
    let mut report = RunReport::default();

    for record in parse_input(input) {
        match record {
            InputRecord::Skipped {
                line_number,
                raw,
                reason,
            } => {
                report.record_row_skipped(line_number, &raw, reason);
            }
            InputRecord::Malformed {
                line_number,
                raw,
                reason,
            } => {
                report.record_row_malformed(line_number, &raw, reason);
            }
            InputRecord::Data(row) => {
                // 繰り返しヘッダは顧客レコードではなく汚染データなので、
                // ログ上で見えるようスキップ行として記録します。
                if is_header_row(&row.fields) {
                    report.record_row_skipped(
                        row.line_number,
                        &row.raw,
                        "embedded header row inside the dataset",
                    );
                    continue;
                }

                report.data_rows_seen += 1;

                let mut cleaned = Vec::with_capacity(HEADER_ROW.len());
                let mut row_has_failure = false;

                for column in Column::ALL {
                    let raw_value = row.fields[column.index()].as_str();
                    let result = columns::process(column, raw_value);
                    if result.disposition.is_failed() {
                        row_has_failure = true;
                    }
                    report.record_field(row.line_number, column, raw_value, &result);
                    cleaned.push(result.value);
                }

                if row_has_failure {
                    report.rows_with_failures += 1;
                }

                report.rows_written += 1;
                cleaned_rows.push(cleaned);
            }
        }
    }

    FormatRun {
        cleaned_rows,
        report,
    }
}

/// 解析済み行がファイル中に混入した重複ヘッダかどうかを返します。
fn is_header_row(fields: &[String]) -> bool {
    fields.len() == HEADER_ROW.len()
        && fields
            .iter()
            .zip(HEADER_ROW.iter())
            .all(|(field, header)| crate::common::clean_text(field).eq_ignore_ascii_case(header))
}

#[cfg(test)]
mod tests {
    use super::format_dataset;

    #[test]
    fn skips_embedded_header_rows() {
        let input = "CustomerID,full_name,email,phone,address_line,city,region,postal_code,country,birth_date,signup_date,last_purchase_date,status,tier,preferred_language,marketing_opt_in,total_spend,order_count,notes\n\
CustomerID,full_name,email,phone,address_line,city,region,postal_code,country,birth_date,signup_date,last_purchase_date,status,tier,preferred_language,marketing_opt_in,total_spend,order_count,notes\n\
CUST1,Alice Smith,alice@example.com,090-1111-2222,1 Main St,Tokyo,Tokyo,1000001,Japan,1980-01-01,2020-01-01,2024-01-01,active,gold,en,1,10,2,test\n";
        let run = format_dataset(input);
        assert_eq!(run.report.rows_written, 1);
        assert_eq!(run.report.skipped_rows, 2);
    }
}
