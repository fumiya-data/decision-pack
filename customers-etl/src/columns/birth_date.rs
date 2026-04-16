//! `birth_date` 列の整形処理です。

use crate::common::{clean_text, parse_flexible_date};
use crate::report::FieldResult;

/// 生年月日を解析し、正規形 `YYYY-MM-DD` で出力します。
pub fn process(raw: &str) -> FieldResult {
    let cleaned = clean_text(raw);
    if cleaned.is_empty() {
        return FieldResult::empty();
    }

    match parse_flexible_date(&cleaned) {
        Ok(date) => FieldResult::success(date.format("%Y-%m-%d").to_string()),
        Err(reason) => FieldResult::failure(cleaned, reason),
    }
}
