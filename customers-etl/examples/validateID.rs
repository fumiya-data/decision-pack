//! `CustomerID` 列だけを単独検証するサンプルです。
//!
//! 本体フォーマッタは全列を処理しますが、ID 規則だけを調整したいときや
//! `CustomerID` 専用処理を直接見せたいときにはこの例が役立ちます。

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use customers_etl::columns::customer_id;
use customers_etl::csv_input::{InputRecord, parse_input};

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("data")
        .join("customers")
        .join("raw")
        .join("raw_customers_5000.csv");
    let input = fs::read_to_string(&input_path)?;
    let mut canonical = 0usize;
    let mut needs_cleanup = Vec::new();
    let mut invalid = Vec::new();

    for record in parse_input(&input) {
        let InputRecord::Data(row) = record else {
            continue;
        };

        // 正規ヘッダ行と、寛容パーサが通常データとして拾った重複ヘッダ行は飛ばします。
        if row.fields[0].trim().eq_ignore_ascii_case("CustomerID") {
            continue;
        }

        let raw_id = row.fields[0].as_str();
        let result = customer_id::process(raw_id);
        match result.reason {
            Some(reason) if result.disposition.is_failed() => {
                invalid.push((row.line_number, raw_id.to_string(), reason));
            }
            _ if result.disposition.is_empty() => invalid.push((
                row.line_number,
                raw_id.to_string(),
                "CustomerID is required".to_string(),
            )),
            _ if result.value == raw_id => canonical += 1,
            _ => needs_cleanup.push((row.line_number, raw_id.to_string(), result.value)),
        }
    }

    println!("CustomerID validation summary");
    println!("  canonical: {}", canonical);
    println!("  needs cleanup: {}", needs_cleanup.len());
    println!("  invalid: {}", invalid.len());

    if !needs_cleanup.is_empty() {
        println!("\nSample IDs that can be normalized:");
        for (line_no, raw, normalized) in needs_cleanup.iter().take(10) {
            println!("  line {line_no}: `{raw}` -> `{normalized}`");
        }
    }

    if !invalid.is_empty() {
        println!("\nSample invalid IDs:");
        for (line_no, raw, reason) in invalid.iter().take(10) {
            println!("  line {line_no}: `{raw}` ({reason})");
        }
    }

    Ok(())
}
