//! `status` 列の整形処理です。

use crate::common::clean_text;
use crate::report::FieldResult;

/// 状態値の揺れを、サポート対象の小さな正規語彙へ対応付けます。
pub fn process(raw: &str) -> FieldResult {
    let cleaned = clean_text(raw);
    if cleaned.is_empty() {
        return FieldResult::failure(String::new(), "status is required");
    }

    let canonical = match cleaned.to_ascii_lowercase().as_str() {
        "active" => "active",
        "inactive" => "inactive",
        "pending" => "pending",
        "banned" => "banned",
        _ => return FieldResult::failure(cleaned, "unsupported status value"),
    };

    FieldResult::success(canonical)
}
