//! `CustomerID` 列の整形処理です。

use crate::common::{clean_text, normalize_width};
use crate::report::FieldResult;

/// 顧客識別子を正規形 `CUST000000` へそろえます。
///
/// 元ファイルには大文字小文字の揺れ、任意ハイフン、BOM ノイズ、全角数字が
/// 混在するため、まず強めに正規化してから意味的な構造を検証します。
pub fn process(raw: &str) -> FieldResult {
    let cleaned = clean_text(raw);
    if cleaned.is_empty() {
        return FieldResult::failure(String::new(), "CustomerID is required");
    }

    if cleaned.eq_ignore_ascii_case("CustomerID") {
        return FieldResult::failure(cleaned, "embedded header marker");
    }

    let normalized = normalize_width(&cleaned);
    let upper = normalized.to_ascii_uppercase();
    // `CUST123` と `CUST-123` の両方を許容し、古いエクスポートも
    // 1 つの正規表現へそろえられるようにします。
    let digits = if upper.starts_with("CUST-") {
        normalized[5..].trim()
    } else if upper.starts_with("CUST") {
        normalized[4..].trim()
    } else {
        return FieldResult::failure(normalized, "missing CUST prefix");
    };

    if digits.is_empty() {
        return FieldResult::failure(normalized, "missing numeric suffix");
    }

    if !digits.chars().all(|ch| ch.is_ascii_digit()) {
        return FieldResult::failure(normalized, "numeric suffix must contain only digits");
    }

    if digits.len() > 6 {
        return FieldResult::failure(normalized, "numeric suffix must be at most 6 digits");
    }

    match digits.parse::<u32>() {
        Ok(number) => FieldResult::success(format!("CUST{number:06}")),
        Err(_) => FieldResult::failure(normalized, "numeric suffix is out of range"),
    }
}
