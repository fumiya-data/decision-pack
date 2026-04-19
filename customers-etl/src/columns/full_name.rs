//! `full_name` 列の整形処理です。

use crate::common::{clean_text, title_case};
use crate::report::FieldResult;

/// 氏名を整った `First Last` 形式へ正規化します。
///
/// 前後空白を除去し、`Last, First` 形を並べ替え、他列データの混入を示す
/// 明らかな文字を弾きます。
pub fn process(raw: &str) -> FieldResult {
    let cleaned = clean_text(raw);
    if cleaned.is_empty() {
        return FieldResult::failure(String::new(), "full_name is required");
    }

    if cleaned.eq_ignore_ascii_case("full_name") {
        return FieldResult::failure(cleaned, "embedded header marker");
    }

    let comma_count = cleaned.matches(',').count();
    let reordered = match comma_count {
        0 => cleaned,
        1 => {
            let (last, first) = match cleaned.split_once(',') {
                Some(parts) => parts,
                None => {
                    return FieldResult::failure(cleaned, "invalid comma-delimited name");
                }
            };
            let last = clean_text(last);
            let first = clean_text(first);
            if first.is_empty() || last.is_empty() {
                return FieldResult::failure(cleaned, "comma-delimited name is missing one side");
            }
            format!("{first} {last}")
        }
        _ => {
            return FieldResult::failure(
                cleaned,
                "contains too many commas to be a valid full_name",
            );
        }
    };

    // 数字、メール記号、その他の明白な列ずれ信号は拒否します。
    // ヒンディー語などで使われる結合記号は、スクリプト範囲で明示的に許可します。
    if reordered.chars().any(|ch| !is_allowed_name_char(ch)) {
        return FieldResult::failure(
            reordered,
            "contains characters outside the allowed name set",
        );
    }

    let parts = reordered
        .split_whitespace()
        .filter(|part| part.chars().any(char::is_alphabetic))
        .count();
    let non_ascii_alpha_count = reordered
        .chars()
        .filter(|ch| ch.is_alphabetic() && !ch.is_ascii())
        .count();
    if parts < 2 && non_ascii_alpha_count < 2 {
        return FieldResult::failure(reordered, "expected at least a first and last name");
    }

    FieldResult::success(title_case(&reordered))
}

fn is_allowed_name_char(ch: char) -> bool {
    ch.is_whitespace()
        || matches!(ch, '\'' | '-' | '.')
        || ch.is_alphabetic()
        || is_supported_combining_mark(ch)
}

fn is_supported_combining_mark(ch: char) -> bool {
    let code = ch as u32;
    ((0x0900..=0x097f).contains(&code)
        || (0x1cd0..=0x1cff).contains(&code)
        || (0xa8e0..=0xa8ff).contains(&code))
        && !ch.is_numeric()
        && !matches!(ch, '।' | '॥')
}
