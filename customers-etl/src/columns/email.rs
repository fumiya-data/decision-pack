//! `email` 列の整形処理です。

use crate::common::{clean_text, is_placeholder};
use crate::report::FieldResult;

/// メールアドレスを正規化して検証します。
///
/// 完全な RFC パーサではなく、意図的に簡潔な構造検証に留めています。
/// これにより、汚れたエクスポートに合わせつつ、明らかな列ずれや不正な
/// アドレスは捕捉できます。
pub fn process(raw: &str) -> FieldResult {
    let compact = clean_text(raw).replace(' ', "").to_ascii_lowercase();
    if compact.is_empty() || is_placeholder(&compact) {
        return FieldResult::empty();
    }

    let mut parts = compact.split('@');
    let local = parts.next().unwrap_or("");
    let domain = parts.next().unwrap_or("");
    if parts.next().is_some() || local.is_empty() || domain.is_empty() {
        return FieldResult::failure(compact, "email must contain exactly one @");
    }

    if !domain.contains('.') {
        return FieldResult::failure(compact, "email domain must contain a dot");
    }

    if compact.contains("..") {
        return FieldResult::failure(compact, "email must not contain consecutive dots");
    }

    let local_ok = local
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '%' | '+' | '-'));
    let domain_ok = domain
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-'));
    if !local_ok || !domain_ok {
        return FieldResult::failure(compact, "email contains unsupported characters");
    }

    if domain
        .split('.')
        .any(|label| label.is_empty() || label.starts_with('-') || label.ends_with('-'))
    {
        return FieldResult::failure(compact, "email domain labels are malformed");
    }

    FieldResult::success(compact)
}
