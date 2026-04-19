# full サンプルのローカル専用方針を明文化

- `50,000顧客 / 100商品 / 150,000注文` の full サンプルは、開発環境での評価と検証にのみ使う方針を固定した
- full 規模の raw CSV、整形済み CSV、受注 CSV、評価出力 CSV は Git と GitHub に反映しない
- `.gitignore` に `*50000*` と `*150000*` を含む full 規模データの除外ルールを追加した
- `docs/specs/sample-dataset-spec.md` と `docs/specs/non-functional-requirements.md` に、生成器・seed・仕様書・metadata を正本とする方針を追記した
- `git status --ignored` で `data/customers/raw/raw_customers_50000_multilingual.csv` と metadata が ignore 対象として表示されることを確認した
