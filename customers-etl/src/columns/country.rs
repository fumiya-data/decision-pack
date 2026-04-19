//! `country` 列の整形処理です。

use crate::common::{clean_text, title_case};
use crate::report::FieldResult;

/// 国名の別名や国コードを正規表示名へ対応付けます。
pub fn process(raw: &str) -> FieldResult {
    let cleaned = clean_text(raw);
    if cleaned.is_empty() {
        return FieldResult::failure(String::new(), "country is required");
    }

    let key = cleaned
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();

    let canonical = match key.as_str() {
        "japan" | "jp" | "jpn" => "Japan".to_string(),
        "unitedstates" | "us" | "usa" => "United States".to_string(),
        "unitedkingdom" | "uk" | "gb" => "United Kingdom".to_string(),
        "canada" | "ca" => "Canada".to_string(),
        "australia" | "au" => "Australia".to_string(),
        "germany" | "de" => "Germany".to_string(),
        "france" => "France".to_string(),
        "india" | "in" | "ind" => "India".to_string(),
        "china" | "cn" | "prc" => "China".to_string(),
        "singapore" | "sg" => "Singapore".to_string(),
        "spain" => "Spain".to_string(),
        _ => title_case(&cleaned),
    };

    FieldResult::success(canonical)
}
