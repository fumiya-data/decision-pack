//! `notes` 列の整形処理です。

use crate::common::clean_multiline_text;
use crate::report::FieldResult;

/// 自由記述のメモをコンパクトな 1 行表現へ畳み込みます。
pub fn process(raw: &str) -> FieldResult {
    let cleaned = clean_multiline_text(raw);
    if cleaned.is_empty() {
        return FieldResult::empty();
    }

    FieldResult::success(cleaned)
}
