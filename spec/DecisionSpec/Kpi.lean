import DecisionSpec.Common

namespace DecisionSpec

/-- KPI 集約の最小仕様。最低資金残高と累積欠品を保持する。 -/
structure KpiSummary where
  minCash : Yen
  totalStockout : Qty
deriving Repr

/-- 観測した資金がより低ければ `minCash` を更新する。 -/
def ObserveCash (summary : KpiSummary) (cash : Yen) : KpiSummary :=
  if cash < summary.minCash then
    { summary with minCash := cash }
  else
    summary

/-- 欠品数量を累積加算する。 -/
def AddStockout (summary : KpiSummary) (qty : Qty) : KpiSummary :=
  { summary with totalStockout := summary.totalStockout + qty }

/-- `ObserveCash` は `minCash` を増やさない。 -/
theorem observe_cash_never_increases
  (summary : KpiSummary)
  (cash : Yen) :
  (ObserveCash summary cash).minCash ≤ summary.minCash := by
  by_cases h : cash < summary.minCash
  · simp [ObserveCash, h, Int.le_of_lt h]
  · simp [ObserveCash, h]

/-- `AddStockout` は累積欠品数量を減らさない。 -/
theorem add_stockout_monotone
  (summary : KpiSummary)
  (qty : Qty) :
  summary.totalStockout ≤ (AddStockout summary qty).totalStockout := by
  simp [AddStockout, Nat.le_add_right]

end DecisionSpec