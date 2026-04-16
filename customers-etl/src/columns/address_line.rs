//! `address_line` 列の整形処理です。

use crate::common::{clean_text, is_placeholder, normalize_comma_spacing};
use crate::report::FieldResult;

/// 地理情報の推測までは行わず、住所の記号と空白を整えます。
pub fn process(raw: &str) -> FieldResult {
    let cleaned = clean_text(raw);
    if cleaned.is_empty() || is_placeholder(&cleaned) {
        return FieldResult::empty();
    }

    FieldResult::success(normalize_comma_spacing(&cleaned))
}
