//! `total_spend` 列の整形処理です。

use crate::common::{clean_text, is_placeholder, normalize_width};
use crate::report::FieldResult;

/// 金額値を小数第 2 位までの単純な 10 進文字列へ正規化します。
///
/// 入力にはカンマ、空白、通貨記号が混じるため、解析前に取り除いて
/// 整形済み出力が数値のまま保てるようにします。
pub fn process(raw: &str) -> FieldResult {
    let cleaned = clean_text(raw);
    if cleaned.is_empty() || is_placeholder(&cleaned) {
        return FieldResult::empty();
    }

    let numeric = normalize_width(&cleaned)
        .replace([' ', ','], "")
        .replace(['¥', '$'], "");
    match numeric.parse::<f64>() {
        Ok(value) if value >= 0.0 => FieldResult::success(format!("{value:.2}")),
        Ok(_) => FieldResult::failure(numeric, "total_spend must not be negative"),
        Err(_) => FieldResult::failure(numeric, "total_spend is not numeric"),
    }
}
