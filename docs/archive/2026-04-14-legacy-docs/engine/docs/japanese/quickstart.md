# クイックスタート

## 1. 実行

```bash
cd engine
cargo run
```

期待される出力例:

```text
cash=111000
```

## 2. 何をしているか

- 初期キャッシュ `100000` 円を設定
- 当日のイベントを2件設定
  - 売上 `+15000`
  - 仕入 `-4000`
- `sim::cashflow::cash_one_day` で1日分更新

## 3. テスト

```bash
cargo test
```

`tests/golden_smoke.rs` で在庫・資金・JSON出力の基本挙動を確認できます。
