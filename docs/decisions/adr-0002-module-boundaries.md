# ADR-0002 モジュール境界の固定

## 状態

採用済み

## 背景

実験的に個別に育てていた既存モジュールを協調させて、さらに大きく育てたいため、各モジュール（顧客、購入履歴、在庫、需要予測、意思決定支援、GUI/API）について、各自の責務を明確にし、このADRで固定することによって、後続実装で責務が混ざることを予防したい。

## 決定
各自の担う責務は以下のとおりとする。
- 顧客データの整形処理は `customers-etl`
- 商品、受注、在庫取込は `commerce-etl`
- 購入傾向推定は `purchase-insights`
- 意思決定計算は `decision-engine`
- GUI 向け境界は `app-api`（GUIを薄く保つため）
- GUI は `desktop-ui`
- `decision-engine` には顧客個票を持たせない

## 影響

- 顧客一覧や在庫検索は `decision-engine` の責務にしない。
- GUI は API 越しにデータを取得する。
- 需要予測は `purchase-insights` から `decision-engine` へ集約データとして渡す。
