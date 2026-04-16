//! `marketing_opt_in` 列の整形処理です。

use crate::common::clean_text;
use crate::report::FieldResult;

/// 真偽表現の揺れを `true` / `false` へ対応付けます。
pub fn process(raw: &str) -> FieldResult {
    let cleaned = clean_text(raw);
    if cleaned.is_empty() {
        return FieldResult::failure(String::new(), "marketing_opt_in is required");
    }

    let output = match cleaned.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "y" => "true",
        "0" | "false" | "no" | "n" => "false",
        _ => return FieldResult::failure(cleaned, "unsupported marketing_opt_in value"),
    };

    FieldResult::success(output)
}
