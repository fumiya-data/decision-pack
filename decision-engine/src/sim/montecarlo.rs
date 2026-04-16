/// モンテカルロ出力で使う分位要約です。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonteCarloSummary {
    /// 第 10 パーセンタイルです。
    pub p10: f64,
    /// 第 50 パーセンタイル（中央値）です。
    pub p50: f64,
    /// 第 90 パーセンタイルです。
    pub p90: f64,
}

/// サンプル列から単純な分位要約を構築します。
///
/// 入力が空の場合は `None` を返します。
pub fn summarize_percentiles(samples: &[f64]) -> Option<MonteCarloSummary> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    Some(MonteCarloSummary {
        p10: percentile(&sorted, 0.10),
        p50: percentile(&sorted, 0.50),
        p90: percentile(&sorted, 0.90),
    })
}

/// 昇順ソート済みの列から近似的な分位値を返します。
fn percentile(sorted: &[f64], p: f64) -> f64 {
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[idx]
}
