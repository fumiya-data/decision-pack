# Decision Pack: 製品要件 v0.1

## 1. プロダクトの目的

- 対象: 中小企業（目安10名規模）
- 目的:
  - 30/60/90日先のキャッシュ残高を予測し、資金ショートを回避する
  - 在庫の欠品と過剰在庫を抑え、利益と資金効率を両立する

## 2. 想定ユーザー

- 経営者
- 店長/現場責任者
- バックオフィス（経理・購買）

## 3. 提供価値

- 「いつ資金が不足するか」を日次で可視化
- 「何をどれだけいつ発注すべきか」をシナリオ比較で提示
- 制約違反（資金不足、欠品率超過、発注上限制約超過）を早期警告

## 4. システム構成

- 入力: 最小CSV + 事業ルール設定
- 仕様: Lean（不変条件、目的関数、制約、計算意味）
- エンジン: Rust（決定論シミュレーション、モンテカルロ、制約違反検出）
- 出力: 図表（Pythonライブラリで生成）
- UI: iced（配布しやすいデスクトップ）

## 5. MVP 機能（最初の出荷範囲）

- キャッシュフロー予測
  - ベースライン + 悲観/楽観シナリオ
  - 30/60/90日予測
- 在庫最適化
  - 品目別に欠品数量、在庫日数、発注提案を算出
- 制約違反アラート
  - 予測残高が閾値未満
  - 欠品率が閾値超過
  - 発注額が資金上限超過
- シナリオ比較
  - 需要変動、リードタイム悪化、仕入単価上昇、固定費変動

## 6. CSV 入出力（v0.1）

### 6.1 既存テーブル

- `items`（商品/品目）
- `sales_daily`（日次売上）
- `inventory_daily`（日次在庫）
- `staff`（スタッフ）
- `cashflow_daily`（日次入出金）

### 6.2 追加提案テーブル

- `purchase_orders`（発注残）
  - `po_id,item_id,order_date,due_date,qty,unit_cost,status`
- `ar_ap_terms`（入金/支払条件）
  - `counterparty_id,kind,term_days,closing_rule`
- `scenario_params`（シナリオ係数）
  - `scenario_id,demand_factor,lead_time_factor,cost_factor,fixed_cost_delta`
- `calendar`（営業日）
  - `date,is_business_day`

### 6.3 各CSVの最低限カラム提案

- `items`: `item_id,category,lead_time_days,moq,lot_size,unit_cost,unit_price,safety_stock`
- `sales_daily`: `date,item_id,qty,unit_price`
- `inventory_daily`: `date,item_id,on_hand,on_order`
- `staff`: `staff_id,skill,wage_hour,max_hours_day,max_hours_week`
- `cashflow_daily`: `date,category,amount,direction,counterparty`

### 6.4 データ設計ルール

- 金額単位は円（`Yen`）、数量は非負整数（`Qty`）
- 日付は `Date` として日次粒度で扱う
- 主キー相当（`date`,`item_id` など）を明示
- 欠損時の扱い（0補完/エラー）をバリデーション規則に明記

## 7. 仕様と実装の整合方針

- Lean 仕様を参照基準（正本）とする
- RustエンジンはLeanの不変条件を破らないようテストで担保
- 代表シナリオはゴールデンテスト化し、回帰を検知

## 8. KPI（v0.1）

- キャッシュ最小残高
- 資金ショート発生日
- 欠品数量/欠品率
- 在庫日数（DOH）
- シナリオ別の利益・キャッシュ差分

## 9. 開発ロードマップ

- フェーズ 1: CSV スキーマ固定 + 入力検証
- フェーズ 2: 決定論シミュレーション（在庫・資金）
- フェーズ 3: モンテカルロ（需要/リードタイム変動）
- フェーズ 4: レポート JSON + 図表出力
- フェーズ 5: iced UI 統合（シナリオ編集/比較）

## 10. 非目標（v0.1）

- ERP全面置換
- 自動発注の完全自律化
- 会計仕訳の完全自動化
