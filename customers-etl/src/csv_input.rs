//! 意図的に汚れたエクスポート CSV を復旧する入力側モジュールです。
//!
//! 入力は RFC 4180 に従った綺麗な CSV ではありません。コメント行、重複ヘッダ、
//! 複数行の引用付き `notes`、セミコロン区切り行、末尾付近の壊れた引用が
//! 含まれるため、厳格な CSV リーダーへ直行せず、寛容な行復旧を先に行います。

use crate::schema::HEADER_ROW;

/// 想定列数と一致し、列ごとの整形へ渡せる解析済み行です。
#[derive(Debug, Clone)]
pub struct ParsedRow {
    pub line_number: usize,
    pub raw: String,
    pub fields: Vec<String>,
}

/// 元ファイルから復旧した物理レコードの大分類です。
#[derive(Debug, Clone)]
pub enum InputRecord {
    Data(ParsedRow),
    Skipped {
        line_number: usize,
        raw: String,
        reason: String,
    },
    Malformed {
        line_number: usize,
        raw: String,
        reason: String,
    },
}

#[derive(Debug)]
struct ParseAttempt {
    fields: Vec<String>,
    in_quotes: bool,
}

/// 生の入力文字列から論理レコードを復旧します。
///
/// 物理行単位で処理しつつ、引用が閉じていない場合だけ次行を結合します。
/// これにより正当な複数行 `notes` は保持しつつ、壊れた引用が次の顧客行まで
/// 飲み込むのを防ぎます。
pub fn parse_input(input: &str) -> Vec<InputRecord> {
    let physical_lines: Vec<&str> = input.split('\n').collect();
    let mut records = Vec::new();
    let mut index = 0usize;

    while index < physical_lines.len() {
        let line = physical_lines[index].trim_end_matches('\r');
        let line_number = index + 1;

        // 空行はノイズであり、有用な診断にもならないため黙って捨てます。
        if line.trim().is_empty() {
            index += 1;
            continue;
        }

        // 移行ツールが途中に人間向けコメントを混入させています。
        // それらは不正顧客行ではなく、レポート上のスキップ行として扱います。
        if line.trim_start().starts_with('#') || line.trim_start().starts_with("//") {
            records.push(InputRecord::Skipped {
                line_number,
                raw: line.to_string(),
                reason: "データ行ではありません".to_string(),
            });
            index += 1;
            continue;
        }

        let mut raw = line.to_string();
        let mut delimiter = ',';
        let mut parsed = parse_fields(&raw, delimiter);

        // 引用が閉じるまで物理行を連結します。これにより複数行 `notes` は保持しつつ、
        // 壊れた 1 行レコードは下の不正経路へ落とせます。
        while parsed.in_quotes && index + 1 < physical_lines.len() {
            index += 1;
            raw.push('\n');
            raw.push_str(physical_lines[index].trim_end_matches('\r'));
            parsed = parse_fields(&raw, delimiter);
        }

        // 全体がセミコロン区切りになっている行が 1 件あります。
        // カンマ解析で 1 列しか取れない場合は、即失敗にせず区切り文字を変えて再試行します。
        if parsed.fields.len() == 1 && raw.contains(';') {
            delimiter = ';';
            parsed = parse_fields(&raw, delimiter);
        }

        // `notes` には無引用の区切り文字が混ざることがあります。
        // 列数が多すぎる場合は、あふれた断片を末尾 `notes` に戻します。
        if parsed.fields.len() > HEADER_ROW.len() {
            let separator = if delimiter == ';' { "; " } else { ", " };
            let mut merged = parsed.fields[..HEADER_ROW.len() - 1].to_vec();
            merged.push(parsed.fields[HEADER_ROW.len() - 1..].join(separator));
            parsed.fields = merged;
        }

        // 復旧後も列数が合わないものは真に不正な行なので、出力へねじ込まずログへ残します。
        if parsed.in_quotes || parsed.fields.len() != HEADER_ROW.len() {
            records.push(InputRecord::Malformed {
                line_number,
                raw,
                reason: format!(
                    "想定列数 {} に対して実際は {} 列でした",
                    HEADER_ROW.len(),
                    parsed.fields.len()
                ),
            });
        } else {
            records.push(InputRecord::Data(ParsedRow {
                line_number,
                raw,
                fields: parsed.fields,
            }));
        }

        index += 1;
    }

    records
}

/// 復旧済みレコードを CSV の引用規則を考慮しながら列へ分割します。
///
/// ここでのパーサは専用用途の小さな実装です。エスケープされたダブルクォートと
/// 開始引用前の空白だけを理解すれば、このクレートが扱う汚れたエクスポートには十分です。
fn parse_fields(record: &str, delimiter: char) -> ParseAttempt {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut chars = record.chars().peekable();
    let mut in_quotes = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if in_quotes => {
                if matches!(chars.peek(), Some('"')) {
                    field.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            }
            '"' if field.chars().all(char::is_whitespace) => {
                field.clear();
                in_quotes = true;
            }
            _ if ch == delimiter && !in_quotes => {
                fields.push(field.trim().to_string());
                field.clear();
            }
            _ => field.push(ch),
        }
    }

    fields.push(field.trim().to_string());
    ParseAttempt { fields, in_quotes }
}

#[cfg(test)]
mod tests {
    use super::{InputRecord, parse_input};

    #[test]
    fn preserves_multiline_rows() {
        let input = "CustomerID,full_name,email,phone,address_line,city,region,postal_code,country,birth_date,signup_date,last_purchase_date,status,tier,preferred_language,marketing_opt_in,total_spend,order_count,notes\n\
CUST1,Alice Smith,alice@example.com,09011112222,\"1 Main St\nApt 2\",Tokyo,Tokyo,1000001,Japan,1980-01-01,2020-01-01,2024-01-01,active,gold,en,1,10,2,test\n";
        let records = parse_input(input);
        match &records[1] {
            InputRecord::Data(row) => assert_eq!(row.fields[4], "1 Main St\nApt 2"),
            _ => panic!("expected data row"),
        }
    }

    #[test]
    fn marks_malformed_rows_without_swallowing_next_row() {
        let input = "h1,h2,h3,h4,h5,h6,h7,h8,h9,h10,h11,h12,h13,h14,h15,h16,h17,h18,h19\n\
CUST050009,\"Broken Quote,jq@example.com,555-0000,\"8 Quote St\",Boston,MA,02110,United States,1989-01-01,2024-01-01,2024-02-02,active,silver,en,1,77.77,1,unmatched quote in second column\n\
CUST004121,Yuki Rodriguez,yuki.rodriguez@example.org,+81 46 2825 5956,\"1-1-19 Setagaya, Apt. 1019\",umeda,Fukuoka,7265286,Japan,1969-04-08,2021-09-02,2023-08-05,banned,gold,EN,1,N/A,10,VIP\n";
        let records = parse_input(input);
        assert!(matches!(records[1], InputRecord::Malformed { .. }));
        assert!(matches!(records[2], InputRecord::Data(_)));
    }
}
