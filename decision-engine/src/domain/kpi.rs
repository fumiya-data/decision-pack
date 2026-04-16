use crate::domain::money::{Qty, Yen};

/// シミュレーション集計中に使う最小限の KPI コンテナです。
///
/// `min_cash` は `i64::MAX` から始まり、各日について
/// [`KpiSummary::observe_cash`] を呼んで更新します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KpiSummary {
    /// 観測された最小資金残高です。
    pub min_cash: Yen,
    /// 累積した欠品数量です。
    pub total_stockout: Qty,
}

impl Default for KpiSummary {
    fn default() -> Self {
        Self {
            min_cash: i64::MAX,
            total_stockout: 0,
        }
    }
}

impl KpiSummary {
    /// 1 日分の資金を観測し、より低ければ `min_cash` を更新します。
    pub fn observe_cash(&mut self, cash: Yen) {
        if cash < self.min_cash {
            self.min_cash = cash;
        }
    }

    /// 飽和加算で欠品数量を加算します。
    pub fn add_stockout(&mut self, qty: Qty) {
        self.total_stockout = self.total_stockout.saturating_add(qty);
    }
}
