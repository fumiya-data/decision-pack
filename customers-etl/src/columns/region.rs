//! `region` 列の整形処理です。

use crate::common::{clean_text, is_placeholder, title_case};
use crate::report::FieldResult;

/// 州・県・都道府県などの地域値を正規化します。
///
/// 2 文字の英字値はコードとみなして大文字化し、それより長い値は
/// タイトルケースへそろえます。
pub fn process(raw: &str) -> FieldResult {
    let cleaned = clean_text(raw);
    if cleaned.is_empty() || is_placeholder(&cleaned) {
        return FieldResult::empty();
    }

    if cleaned.len() == 2 && cleaned.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return FieldResult::success(cleaned.to_ascii_uppercase());
    }

    FieldResult::success(title_case(&cleaned))
}
