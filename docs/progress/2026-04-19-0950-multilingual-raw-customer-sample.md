# multilingual raw customer sample

## 実施内容

- `customers-etl` に未整形顧客 raw サンプル生成モードを追加する前提のコード変更を開始した
- `preferred_language` に `hi`, `zh`, `zh-CN`, `zh-TW` を追加した
- `country` に `India`, `China` 系の正規化を追加した
- `full_name` は非 ASCII 文字だけで構成される単一トークン名を受け入れられるようにした
- `data/customers/raw/raw_customers_50000_multilingual.csv` を生成した
- `data/customers/raw/raw_customers_50000_multilingual_metadata.json` を生成した

## 生成データの概要

- 有効顧客行: `50,000`
- 重複ヘッダ行: `4`
- 無効行: `240`
- raw 総行数: `50,245`  
  ヘッダ 1 行を含む

## 多言語対応

- 英語名と米国住所
- 日本語名と日本住所
- ヒンディー語名とインド住所
- 中国語名と中国住所

## 補足

- この環境では `link.exe` 不足により `cargo test -p customers-etl` を完走できなかった
- ただし `cargo fmt --all --check` は通しており、raw データファイル自体は生成済み
