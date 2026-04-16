//! 下流の意思決定支援へ渡す顧客セグメント集計です。

use std::collections::BTreeMap;

use chrono::NaiveDate;

use crate::schema::Column;

/// `customer_segment_summary.csv` の 1 行を表します。
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRow {
    pub country: String,
    pub status: String,
    pub tier: String,
    pub customer_count: usize,
    pub marketing_opt_in_true_count: usize,
    pub total_spend_known_count: usize,
    pub total_spend_sum: f64,
    pub total_spend_avg: f64,
    pub order_count_known_count: usize,
    pub order_count_sum: u64,
    pub order_count_avg: f64,
    pub last_purchase_known_count: usize,
}

/// 出力行と除外件数の両方を含む集計結果です。
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSummary {
    pub rows: Vec<SegmentRow>,
    pub excluded_invalid_keys: usize,
}

#[derive(Debug, Default)]
struct SegmentAccumulator {
    customer_count: usize,
    marketing_opt_in_true_count: usize,
    total_spend_known_count: usize,
    total_spend_sum: f64,
    order_count_known_count: usize,
    order_count_sum: u64,
    last_purchase_known_count: usize,
}

impl SegmentAccumulator {
    fn into_row(self, country: String, status: String, tier: String) -> SegmentRow {
        let total_spend_avg = if self.total_spend_known_count == 0 {
            0.0
        } else {
            self.total_spend_sum / self.total_spend_known_count as f64
        };
        let order_count_avg = if self.order_count_known_count == 0 {
            0.0
        } else {
            self.order_count_sum as f64 / self.order_count_known_count as f64
        };

        SegmentRow {
            country,
            status,
            tier,
            customer_count: self.customer_count,
            marketing_opt_in_true_count: self.marketing_opt_in_true_count,
            total_spend_known_count: self.total_spend_known_count,
            total_spend_sum: self.total_spend_sum,
            total_spend_avg,
            order_count_known_count: self.order_count_known_count,
            order_count_sum: self.order_count_sum,
            order_count_avg,
            last_purchase_known_count: self.last_purchase_known_count,
        }
    }
}

/// 下流の意思決定支援が使う安定したセグメント要約を構築します。
pub fn build_segment_summary(cleaned_rows: &[Vec<String>]) -> SegmentSummary {
    let mut grouped: BTreeMap<(String, String, String), SegmentAccumulator> = BTreeMap::new();
    let mut excluded_invalid_keys = 0usize;

    for row in cleaned_rows.iter().skip(1) {
        let Some((country, status, tier)) = valid_key(row) else {
            excluded_invalid_keys += 1;
            continue;
        };

        let entry = grouped.entry((country, status, tier)).or_default();
        entry.customer_count += 1;

        if value(row, Column::MarketingOptIn) == "true" {
            entry.marketing_opt_in_true_count += 1;
        }

        if let Ok(amount) = value(row, Column::TotalSpend).parse::<f64>() {
            entry.total_spend_known_count += 1;
            entry.total_spend_sum += amount;
        }

        if let Ok(order_count) = value(row, Column::OrderCount).parse::<u64>() {
            entry.order_count_known_count += 1;
            entry.order_count_sum += order_count;
        }

        let last_purchase = value(row, Column::LastPurchaseDate);
        if !last_purchase.is_empty() && NaiveDate::parse_from_str(last_purchase, "%Y-%m-%d").is_ok()
        {
            entry.last_purchase_known_count += 1;
        }
    }

    let rows = grouped
        .into_iter()
        .map(|((country, status, tier), acc)| acc.into_row(country, status, tier))
        .collect();

    SegmentSummary {
        rows,
        excluded_invalid_keys,
    }
}

fn valid_key(row: &[String]) -> Option<(String, String, String)> {
    let country = value(row, Column::Country);
    let status = value(row, Column::Status);
    let tier = value(row, Column::Tier);

    if country.is_empty() {
        return None;
    }
    if !matches!(status, "active" | "inactive" | "pending" | "banned") {
        return None;
    }
    if !tier.is_empty() && !matches!(tier, "bronze" | "silver" | "gold" | "platinum") {
        return None;
    }

    Some((country.to_string(), status.to_string(), tier.to_string()))
}

fn value(row: &[String], column: Column) -> &str {
    row.get(column.index()).map(String::as_str).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::build_segment_summary;
    use crate::schema::HEADER_ROW;

    #[test]
    fn groups_rows_by_country_status_and_tier() {
        let rows = vec![
            HEADER_ROW.iter().map(|value| value.to_string()).collect(),
            vec![
                "CUST000001".into(),
                "Alice Smith".into(),
                "alice@example.com".into(),
                "07001338908".into(),
                "2-15-19 Setagaya".into(),
                "Umeda".into(),
                "Fukuoka".into(),
                "265-4235".into(),
                "Japan".into(),
                "1964-03-12".into(),
                "2019-07-04".into(),
                "2020-04-14".into(),
                "active".into(),
                "silver".into(),
                "en-GB".into(),
                "true".into(),
                "457.29".into(),
                "9".into(),
                String::new(),
            ],
            vec![
                "CUST000002".into(),
                "Bob Garcia".into(),
                "bob@example.com".into(),
                "0957015430".into(),
                "9305 Oak Street".into(),
                "Newark".into(),
                "IL".into(),
                "82278-8963".into(),
                "Japan".into(),
                "1968-09-17".into(),
                "2023-10-31".into(),
                String::new(),
                "active".into(),
                "silver".into(),
                "en".into(),
                "false".into(),
                String::new(),
                "1".into(),
                String::new(),
            ],
            vec![
                "CUST000003".into(),
                "Carol Moore".into(),
                "carol@example.org".into(),
                "5346247510".into(),
                "9234 Oak Street".into(),
                "Miami".into(),
                String::new(),
                "42513".into(),
                "United States".into(),
                "2005-09-16".into(),
                String::new(),
                "2026-01-01".into(),
                "??".into(),
                "silver".into(),
                String::new(),
                "true".into(),
                "10.00".into(),
                "3".into(),
                String::new(),
            ],
        ];

        let summary = build_segment_summary(&rows);
        assert_eq!(summary.rows.len(), 1);
        assert_eq!(summary.excluded_invalid_keys, 1);
        assert_eq!(summary.rows[0].customer_count, 2);
        assert_eq!(summary.rows[0].marketing_opt_in_true_count, 1);
        assert_eq!(summary.rows[0].total_spend_known_count, 1);
        assert_eq!(summary.rows[0].order_count_sum, 10);
        assert_eq!(summary.rows[0].last_purchase_known_count, 1);
    }
}
