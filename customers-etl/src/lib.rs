//! CSV 整形パイプラインの共有ライブラリ面です。
//!
//! このクレートは責務が明確になるよう小さなモジュールへ分割しています。
//! `csv_input` は汚れた入力ファイルから行を復旧し、`columns` は列ごとの
//! 正規化規則を持ち、`formatter` は行単位の処理を統括し、`report` は
//! バイナリが表示・保存する成功/失敗要約を扱います。

pub mod columns;
pub mod common;
pub mod config;
pub mod csv_input;
pub mod formatter;
pub mod output;
pub mod persistence;
pub mod report;
pub mod schema;
pub mod summary;

/// 呼び出し側が内部モジュール構成を意識せずに済むよう、
/// エンドツーエンド整形結果を再公開します。
pub use formatter::{FormatRun, format_dataset};
pub use summary::{SegmentRow, SegmentSummary, build_segment_summary};
