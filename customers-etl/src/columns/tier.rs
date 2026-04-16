//! `tier` 列の整形処理です。

use crate::common::{clean_text, is_placeholder};
use crate::report::FieldResult;

/// ロイヤルティ tier の値揺れを正規 tier 集合へ対応付けます。
pub fn process(raw: &str) -> FieldResult {
    let cleaned = clean_text(raw);
    if cleaned.is_empty() || is_placeholder(&cleaned) {
        return FieldResult::empty();
    }

    let canonical = match cleaned.to_ascii_lowercase().as_str() {
        "bronze" => "bronze",
        "silver" | "silv" => "silver",
        "gold" => "gold",
        "platinum" => "platinum",
        _ => return FieldResult::failure(cleaned, "unsupported tier value"),
    };

    FieldResult::success(canonical)
}
