//! # エンジンバイナリ最小実行例
//!
//! このバイナリは、1 日分の資金更新を最小構成で確認するための例です。
//! クレートを初めて触るときの出発点として使うことを想定しています。
//!
//! ## 実行内容
//! - 単純な [`CashState`] を組み立てる
//! - 同日発生のイベント (`sales`, `purchase`) を追加する
//! - [`cash_one_day`] を実行し、結果の資金残高を表示する
//! - スキーマ互換のレポート JSON を `decision-engine/out/` に出力する
//!
//! ## 実行方法
//! ```bash
//! cargo run
//! ```
//!
//! 期待される出力:
//! ```text
//! cash=111000
//! ```

use decision_engine::domain::{CashEvent, CashState};
use decision_engine::report::json::write_json_file;
use decision_engine::report::types::{
    Alert, AlertCode, AlertSeverity, DailyCashPoint, DailyInventoryPoint, KpiReport, ScenarioInfo,
    SimulationReport,
};
use decision_engine::sim::cashflow::cash_one_day;
use decision_engine::sim::inventory::inventory_one_day;
use std::fs;

fn main() {
    let dates = [
        "2026-03-05",
        "2026-03-06",
        "2026-03-07",
        "2026-03-08",
        "2026-03-09",
        "2026-03-10",
        "2026-03-11",
    ];

    let daily_inflow = [120_000_i64, 90_000, 70_000, 60_000, 75_000, 82_000, 100_000];
    let daily_outflow = [85_000_i64, 110_000, 90_000, 80_000, 85_000, 92_000, 95_000];
    let daily_demand = [8_u32, 6, 7, 2, 5, 4, 6];
    let daily_arrivals = [0_u32, 0, 0, 12, 0, 0, 0];

    let mut cash_state = CashState { cash: 1_000_000 };
    let mut on_hand = 12_u32;
    let on_order = 20_u32;

    let mut cash_series = Vec::new();
    let mut inventory_series = Vec::new();
    let mut total_stockout = 0_u32;
    let mut min_cash = i64::MAX;

    for day in 0..dates.len() {
        let events = vec![
            CashEvent {
                due: day as u32,
                amount: daily_inflow[day],
                category: "sales".to_string(),
            },
            CashEvent {
                due: day as u32,
                amount: -daily_outflow[day],
                category: "expense".to_string(),
            },
        ];
        cash_state = cash_one_day(day as u32, cash_state, &events);
        if cash_state.cash < min_cash {
            min_cash = cash_state.cash;
        }

        let inv = inventory_one_day(on_hand, daily_arrivals[day], daily_demand[day]);
        on_hand = inv.next_on_hand;
        total_stockout = total_stockout.saturating_add(inv.stockout);

        cash_series.push(DailyCashPoint {
            date: dates[day].to_string(),
            cash: cash_state.cash,
            inflow: daily_inflow[day],
            outflow: daily_outflow[day],
        });

        inventory_series.push(DailyInventoryPoint {
            date: dates[day].to_string(),
            item_id: "A001".to_string(),
            on_hand,
            on_order,
            demand: daily_demand[day],
            sold: inv.sold,
            stockout: inv.stockout,
        });
    }

    println!("cash={}", cash_state.cash);

    let report = SimulationReport {
        schema_version: "v0.1".to_string(),
        generated_at: "2026-03-05T13:30:00+09:00".to_string(),
        scenario: ScenarioInfo {
            id: "baseline".to_string(),
            name: "ベースライン".to_string(),
            description: Some("バイナリ最小実行例の基本シナリオ".to_string()),
        },
        horizon_days: dates.len() as u32,
        currency: "JPY".to_string(),
        kpi: KpiReport {
            min_cash,
            first_cash_shortfall_date: None,
            total_stockout_qty: total_stockout,
            stockout_rate: 0.021,
            days_on_hand_avg: 12.4,
        },
        alerts: vec![Alert {
            code: AlertCode::StockoutRateBreach,
            severity: AlertSeverity::Warn,
            date: "2026-03-08".to_string(),
            item_id: Some("A001".to_string()),
            message: "欠品率の閾値を超過しました".to_string(),
        }],
        cash_series,
        inventory_series,
    };

    fs::create_dir_all("out").expect("failed to create output directory");
    write_json_file(&report, "out/simulation_report_v0.1.json")
        .expect("failed to write simulation report json");
}
