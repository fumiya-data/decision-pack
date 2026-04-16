use std::fs;
use std::path::Path;

use crate::report::types::{Alert, DailyCashPoint, DailyInventoryPoint, SimulationReport};

/// `SimulationReport` を `simulation_report_v0.1` JSON へシリアライズします。
pub fn to_json(report: &SimulationReport) -> String {
    let alerts = report
        .alerts
        .iter()
        .map(alert_to_json)
        .collect::<Vec<_>>()
        .join(",");

    let cash = report
        .cash_series
        .iter()
        .map(cash_to_json)
        .collect::<Vec<_>>()
        .join(",");

    let inventory = report
        .inventory_series
        .iter()
        .map(inventory_to_json)
        .collect::<Vec<_>>()
        .join(",");

    let desc = report
        .scenario
        .description
        .as_ref()
        .map(|s| format!(r#","description":"{}""#, escape_json(s)))
        .unwrap_or_default();

    let shortfall = report
        .kpi
        .first_cash_shortfall_date
        .as_ref()
        .map(|d| format!(r#""{}""#, escape_json(d)))
        .unwrap_or_else(|| "null".to_string());

    format!(
        concat!(
            r#"{{"schema_version":"{}","generated_at":"{}","scenario":{{"id":"{}","name":"{}"{} }},"#,
            r#""horizon_days":{},"currency":"{}","kpi":{{"min_cash":{},"first_cash_shortfall_date":{},"#,
            r#""total_stockout_qty":{},"stockout_rate":{},"days_on_hand_avg":{}}},"#,
            r#""alerts":[{}],"cash_series":[{}],"inventory_series":[{}]}}"#
        ),
        escape_json(&report.schema_version),
        escape_json(&report.generated_at),
        escape_json(&report.scenario.id),
        escape_json(&report.scenario.name),
        desc,
        report.horizon_days,
        escape_json(&report.currency),
        report.kpi.min_cash,
        shortfall,
        report.kpi.total_stockout_qty,
        report.kpi.stockout_rate,
        report.kpi.days_on_hand_avg,
        alerts,
        cash,
        inventory
    )
}

/// `SimulationReport` を整形済み JSON へシリアライズします。
pub fn to_pretty_json(report: &SimulationReport) -> String {
    pretty_json(&to_json(report))
}

/// レポート JSON を指定パスへ書き出します。
pub fn write_json_file(report: &SimulationReport, path: impl AsRef<Path>) -> std::io::Result<()> {
    fs::write(path, to_pretty_json(report))
}

fn alert_to_json(a: &Alert) -> String {
    let item = a
        .item_id
        .as_ref()
        .map(|id| format!(r#","item_id":"{}""#, escape_json(id)))
        .unwrap_or_else(|| r#","item_id":null"#.to_string());

    format!(
        r#"{{"code":"{}","severity":"{}","date":"{}"{},"message":"{}"}}"#,
        a.code.as_str(),
        a.severity.as_str(),
        escape_json(&a.date),
        item,
        escape_json(&a.message)
    )
}

fn cash_to_json(p: &DailyCashPoint) -> String {
    format!(
        r#"{{"date":"{}","cash":{},"inflow":{},"outflow":{}}}"#,
        escape_json(&p.date),
        p.cash,
        p.inflow,
        p.outflow
    )
}

fn inventory_to_json(p: &DailyInventoryPoint) -> String {
    format!(
        r#"{{"date":"{}","item_id":"{}","on_hand":{},"on_order":{},"demand":{},"sold":{},"stockout":{}}}"#,
        escape_json(&p.date),
        escape_json(&p.item_id),
        p.on_hand,
        p.on_order,
        p.demand,
        p.sold,
        p.stockout
    )
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn pretty_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + input.len() / 2);
    let mut indent = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for ch in input.chars() {
        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => {
                in_string = true;
                out.push(ch);
            }
            '{' | '[' => {
                out.push(ch);
                out.push('\n');
                indent += 1;
                out.push_str(&"  ".repeat(indent));
            }
            '}' | ']' => {
                out.push('\n');
                indent = indent.saturating_sub(1);
                out.push_str(&"  ".repeat(indent));
                out.push(ch);
            }
            ',' => {
                out.push(ch);
                out.push('\n');
                out.push_str(&"  ".repeat(indent));
            }
            ':' => {
                out.push_str(": ");
            }
            c if c.is_whitespace() => {}
            _ => out.push(ch),
        }
    }
    out.push('\n');
    out
}
