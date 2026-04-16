//! `city` 列の整形処理です。

use crate::common::{clean_text, is_placeholder, title_case};
use crate::report::FieldResult;

/// 市区町村名をコンパクトなタイトルケースへ正規化します。
pub fn process(raw: &str) -> FieldResult {
    let cleaned = clean_text(raw);
    if cleaned.is_empty() || is_placeholder(&cleaned) {
        return FieldResult::empty();
    }

    FieldResult::success(title_case(&cleaned))
}
