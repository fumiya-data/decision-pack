//! `phone` 列の整形処理です。

use crate::common::{clean_text, is_placeholder, normalize_width};
use crate::report::FieldResult;

/// 電話番号を、先頭の任意 `+` と数字だけの形へ畳み込みます。
///
/// エクスポートには記号の揺れ、全角数字、実際にはメールがずれ込んだ値まで
/// 混在します。桁数ガードによって妥当な電話番号と列ずれ値を見分けます。
pub fn process(raw: &str) -> FieldResult {
    let cleaned = clean_text(raw);
    if cleaned.is_empty() || is_placeholder(&cleaned) {
        return FieldResult::empty();
    }

    let normalized = normalize_width(&cleaned);
    let mut out = String::new();

    for ch in normalized.chars() {
        if ch == '+' && out.is_empty() {
            out.push(ch);
        } else if ch.is_ascii_digit() {
            out.push(ch);
        }
    }

    let digits = out.strip_prefix('+').unwrap_or(&out);
    if digits.len() < 7 || digits.len() > 15 {
        return FieldResult::failure(out, "phone number must have 7 to 15 digits");
    }

    FieldResult::success(out)
}
