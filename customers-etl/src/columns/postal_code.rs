//! `postal_code` 列の整形処理です。

use crate::common::{clean_text, is_placeholder, normalize_width};
use crate::report::FieldResult;

/// 可能な範囲で国別の形を保ちながら郵便番号を正規化します。
///
/// 日本の数値郵便番号や ZIP+4 はハイフンを振り直し、英国型のような
/// 英数字混在形は大文字化だけ行って基本形を保ちます。
pub fn process(raw: &str) -> FieldResult {
    let cleaned = clean_text(raw);
    if cleaned.is_empty() || is_placeholder(&cleaned) {
        return FieldResult::empty();
    }

    let normalized = normalize_width(&cleaned)
        .replace('〒', "")
        .trim()
        .to_string();
    let collapsed = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
    let digits_only: String = collapsed.chars().filter(|ch| ch.is_ascii_digit()).collect();

    // 純粋な数字列は一般的な正規パターンへ組み替えられます。
    if collapsed.chars().all(|ch| ch.is_ascii_digit() || ch == '-') {
        return match digits_only.len() {
            7 => FieldResult::success(format!("{}-{}", &digits_only[..3], &digits_only[3..])),
            9 => FieldResult::success(format!("{}-{}", &digits_only[..5], &digits_only[5..])),
            _ => FieldResult::success(collapsed),
        };
    }

    let alnum_space_hyphen = collapsed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch.is_whitespace() || ch == '-');
    if !alnum_space_hyphen {
        return FieldResult::failure(collapsed, "postal_code contains unsupported characters");
    }

    FieldResult::success(collapsed.to_ascii_uppercase())
}
