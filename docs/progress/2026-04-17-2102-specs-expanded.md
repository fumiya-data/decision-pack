# 2026-04-17 21:02 specs 詳細化

## 完了済み

- `docs/specs/product-scope.md` を全面的に詳細化した
- `docs/specs/functional-requirements.md` をモジュール別、画面別、API 別に詳細化した
- `docs/specs/domain-model.md` を主要概念、境界、不変条件まで含めて詳細化した
- `docs/specs/data-contracts.md` を CSV、DB、API、レポート JSON 契約の粒度で詳細化した
- `docs/specs/non-functional-requirements.md` を運用、監査、性能、DB 方針、検証方針まで含めて詳細化した

## 次にやること

1. `specs` と `architecture` の間で重複しすぎている説明がないか見直す
2. `app-api` と `db/migrations` の具体的な雛形へ進む
3. `decision-engine` のレポート JSON 拡張項目をフィールド単位へ落とす
