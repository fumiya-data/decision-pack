use crate::domain::date::Date;
use crate::domain::{CashEvent, CashState};

/// 1 日分の資金更新を適用します。
///
/// 日次差分には `due` が `today` と一致するイベントだけを含めます。
pub fn cash_one_day(today: Date, st: CashState, events: &[CashEvent]) -> CashState {
    let delta = events
        .iter()
        .filter(|e| e.due == today)
        .fold(0_i64, |acc, e| acc + e.amount);
    CashState {
        cash: st.cash + delta,
    }
}

/// イベント列連結に対する日次更新の加法性を検査します。
///
/// Lean の次の定理に対応します。
/// `CashOneDay(today, st, a ++ b) == CashOneDay(today, CashOneDay(today, st, a), b)`.
pub fn cash_additive_holds(today: Date, st: CashState, a: &[CashEvent], b: &[CashEvent]) -> bool {
    let mut ab = Vec::with_capacity(a.len() + b.len());
    ab.extend_from_slice(a);
    ab.extend_from_slice(b);

    cash_one_day(today, st, &ab) == cash_one_day(today, cash_one_day(today, st, a), b)
}
