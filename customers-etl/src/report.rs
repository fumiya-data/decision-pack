//! 列単位・行単位の整形結果を集約するレポート基盤です。

use crate::schema::Column;

/// 1 つのフィールド値を処理したときの大まかな結果です。
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FieldDisposition {
    Success,
    Empty,
    Failed,
}

impl FieldDisposition {
    /// 安全に正規化できず、失敗として扱う場合に `true` を返します。
    pub const fn is_failed(self) -> bool {
        matches!(self, FieldDisposition::Failed)
    }

    /// 失敗ではなく、意図的に空値扱いした場合に `true` を返します。
    /// 任意列やプレースホルダ値で使います。
    pub const fn is_empty(self) -> bool {
        matches!(self, FieldDisposition::Empty)
    }
}

/// 個々の列処理器が返す最終値と状態です。
///
/// 列検証に失敗しても、整形済み CSV には正規化後 `value` を残します。
/// これにより issue log では生値とベストエフォート結果を並べて確認できます。
#[derive(Debug, Clone)]
pub struct FieldResult {
    pub value: String,
    pub disposition: FieldDisposition,
    pub reason: Option<String>,
}

impl FieldResult {
    /// 成功したフィールド結果を構築します。
    pub fn success(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            disposition: FieldDisposition::Success,
            reason: None,
        }
    }

    /// 任意列やプレースホルダ向けの明示的な空値結果を構築します。
    pub fn empty() -> Self {
        Self {
            value: String::new(),
            disposition: FieldDisposition::Empty,
            reason: None,
        }
    }

    /// 失敗結果を構築します。
    /// それでも cleaned CSV に書く代替値は保持します。
    pub fn failure(value: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            disposition: FieldDisposition::Failed,
            reason: Some(reason.into()),
        }
    }
}

/// 実行全体を通した 1 列ぶんの集計値です。
#[derive(Debug, Clone, Default)]
pub struct ColumnStats {
    pub success: usize,
    pub empty: usize,
    pub failed: usize,
}

/// issue log に出力するレポート項目の種別です。
#[derive(Debug, Clone, Copy)]
pub enum IssueKind {
    RowSkipped,
    RowMalformed,
    FieldFailed,
}

impl IssueKind {
    /// CSV issue log に書く安定文字列です。
    pub const fn as_str(self) -> &'static str {
        match self {
            IssueKind::RowSkipped => "row_skipped",
            IssueKind::RowMalformed => "row_malformed",
            IssueKind::FieldFailed => "field_failed",
        }
    }
}

/// issue log に載せる 1 件の確認項目です。
///
/// フィールド失敗は対象列を持ちます。コメント行、重複ヘッダ、不正行のような
/// 行単位問題は `column` を空にし、出力 CSV では特別な `__row__` を使います。
#[derive(Debug, Clone)]
pub struct IssueRecord {
    pub line_number: usize,
    pub kind: IssueKind,
    pub column: Option<Column>,
    pub raw_value: String,
    pub output_value: String,
    pub reason: String,
}

/// 整形実行全体の要約です。
///
/// `FormatRun` は整形済み行を持ち、`RunReport` は何行処理し、何行をスキップし、
/// 何行が不正で、各列がどう振る舞ったかという運用視点を持ちます。
#[derive(Debug, Clone)]
pub struct RunReport {
    pub data_rows_seen: usize,
    pub rows_written: usize,
    pub rows_with_failures: usize,
    pub skipped_rows: usize,
    pub malformed_rows: usize,
    column_stats: Vec<ColumnStats>,
    pub issues: Vec<IssueRecord>,
}

impl Default for RunReport {
    fn default() -> Self {
        Self {
            data_rows_seen: 0,
            rows_written: 0,
            rows_with_failures: 0,
            skipped_rows: 0,
            malformed_rows: 0,
            column_stats: vec![ColumnStats::default(); crate::schema::HEADER_ROW.len()],
            issues: Vec::new(),
        }
    }
}

impl RunReport {
    /// 1 フィールドの結果を記録し、必要なら集計値と issue log を更新します。
    pub fn record_field(
        &mut self,
        line_number: usize,
        column: Column,
        raw_value: &str,
        result: &FieldResult,
    ) {
        let stats = &mut self.column_stats[column.index()];
        match result.disposition {
            FieldDisposition::Success => stats.success += 1,
            FieldDisposition::Empty => stats.empty += 1,
            FieldDisposition::Failed => {
                stats.failed += 1;
                self.issues.push(IssueRecord {
                    line_number,
                    kind: IssueKind::FieldFailed,
                    column: Some(column),
                    raw_value: raw_value.to_string(),
                    output_value: result.value.clone(),
                    reason: result
                        .reason
                        .clone()
                        .unwrap_or_else(|| "列整形に失敗しました".to_string()),
                });
            }
        }
    }

    /// コメント行や混入ヘッダのように、列処理前に意図的にスキップした行を記録します。
    pub fn record_row_skipped(
        &mut self,
        line_number: usize,
        raw_value: &str,
        reason: impl Into<String>,
    ) {
        self.skipped_rows += 1;
        self.issues.push(IssueRecord {
            line_number,
            kind: IssueKind::RowSkipped,
            column: None,
            raw_value: raw_value.to_string(),
            output_value: String::new(),
            reason: reason.into(),
        });
    }

    /// 想定スキーマ形状へ解析できなかった行を記録します。
    pub fn record_row_malformed(
        &mut self,
        line_number: usize,
        raw_value: &str,
        reason: impl Into<String>,
    ) {
        self.malformed_rows += 1;
        self.issues.push(IssueRecord {
            line_number,
            kind: IssueKind::RowMalformed,
            column: None,
            raw_value: raw_value.to_string(),
            output_value: String::new(),
            reason: reason.into(),
        });
    }

    /// 指定列の累積集計値を返します。
    pub fn column_stats(&self, column: Column) -> &ColumnStats {
        &self.column_stats[column.index()]
    }
}
