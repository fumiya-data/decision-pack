//! `order_count` 列の整形処理です。

use crate::common::clean_text;
use crate::report::FieldResult;

/// 注文回数を非負整数として解析します。
pub fn process(raw: &str) -> FieldResult {
    let cleaned = clean_text(raw);
    if cleaned.is_empty() {
        return FieldResult::empty();
    }

    match cleaned.parse::<u32>() {
        Ok(value) => FieldResult::success(value.to_string()),
        Err(_) => FieldResult::failure(cleaned, "order_count must be a non-negative integer"),
    }
}
