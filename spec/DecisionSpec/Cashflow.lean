import DecisionSpec.Common

namespace DecisionSpec

/-- 入出金予定イベント。dueが計上日、amountは正で入金・負で出金を表す。 -/
structure CashEvent where
  due : Date
  amount : Yen
  category : String
deriving Repr

/-- 1日分の現金更新。指定日のイベントだけを合計し、cashに反映する。 -/
def CashOneDay (today : Date) (st : CashState) (events : List CashEvent) : CashState :=
  let delta := (events.filter (fun e => e.due = today)).foldl (fun acc e => acc + e.amount) 0
  { cash := st.cash + delta }

/-- 連結加法性の不変条件。イベント列を連結して1回更新した結果は、2段階更新と一致する。 -/
theorem cash_additive
  (today : Date) (st : CashState) (a b : List CashEvent) :
  CashOneDay today st (a ++ b) =
    CashOneDay today (CashOneDay today st a) b := by
  have foldl_shift :
      ∀ (xs : List CashEvent) (init : Yen),
        xs.foldl (fun acc e => acc + e.amount) init =
          init + xs.foldl (fun acc e => acc + e.amount) 0 := by
    intro xs
    induction xs with
    | nil =>
        intro init
        simp
    | cons x xs ih =>
        intro init
        calc
          xs.foldl (fun acc e => acc + e.amount) (init + x.amount)
              = (init + x.amount) + xs.foldl (fun acc e => acc + e.amount) 0 := ih (init + x.amount)
          _ = init + (x.amount + xs.foldl (fun acc e => acc + e.amount) 0) := by ac_rfl
          _ = init + xs.foldl (fun acc e => acc + e.amount) x.amount := by rw [ih x.amount]
          _ = init + List.foldl (fun acc e => acc + e.amount) 0 (x :: xs) := by simp [List.foldl]
  cases st with
  | mk cash =>
      simp [CashOneDay, List.filter_append, List.foldl_append, Int.add_assoc]
      exact foldl_shift (List.filter (fun e => e.due = today) b)
        ((List.filter (fun e => e.due = today) a).foldl (fun acc e => acc + e.amount) 0)

end DecisionSpec
