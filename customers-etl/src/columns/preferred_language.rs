//! `preferred_language` 列の整形処理です。

use crate::common::clean_text;
use crate::report::FieldResult;

/// 言語タグを、このエクスポートで扱う小さな許容集合へ正規化します。
pub fn process(raw: &str) -> FieldResult {
    let cleaned = clean_text(raw);
    if cleaned.is_empty() {
        return FieldResult::empty();
    }

    let canonical = cleaned.replace('_', "-").to_ascii_lowercase();
    let output = match canonical.as_str() {
        "en" => "en",
        "en-gb" => "en-GB",
        "en-us" => "en-US",
        "ja" | "jp" => "ja",
        "hi" | "hi-in" => "hi",
        "zh" => "zh",
        "zh-cn" => "zh-CN",
        "zh-tw" => "zh-TW",
        "es" => "es",
        _ => return FieldResult::failure(cleaned, "unsupported language code"),
    };

    FieldResult::success(output)
}
