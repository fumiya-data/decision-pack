use iced::widget::{button, column, container, progress_bar, row, scrollable, text, text_input};
use iced::{Element, Length, Task, Theme};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() -> iced::Result {
    iced::application("Decision Pack UI（Iced）", update, view)
        .theme(|_| Theme::TokyoNight)
        .run_with(|| (App::default(), Task::none()))
}

#[derive(Debug, Clone, Default)]
struct App {
    input_path: String,
    status: String,
    report: Option<Report>,
    loaded_json_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    BrowsePressed,
    LoadPressed,
    OpenCashChart,
    OpenStockoutChart,
    OpenSummary,
    GenerateReportArtifacts,
}

#[derive(Debug, Clone, Deserialize)]
struct Scenario {
    name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Kpi {
    min_cash: i64,
    total_stockout_qty: u32,
    stockout_rate: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct Alert {
    code: String,
    severity: String,
    date: String,
    item_id: Option<String>,
    message: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Report {
    #[allow(dead_code)]
    schema_version: Option<String>,
    generated_at: String,
    scenario: Scenario,
    kpi: Kpi,
    #[serde(default)]
    alerts: Vec<Alert>,
    #[serde(default)]
    cash_series: Vec<CashPoint>,
    #[serde(default)]
    inventory_series: Vec<InventoryPoint>,
}

#[derive(Debug, Clone, Deserialize)]
struct CashPoint {
    date: String,
    cash: i64,
    inflow: i64,
    outflow: i64,
}

#[derive(Debug, Clone, Deserialize)]
struct InventoryPoint {
    date: String,
    item_id: String,
    demand: u32,
    sold: u32,
    stockout: u32,
}

fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::InputChanged(v) => {
            app.input_path = v;
        }
        Message::BrowsePressed => {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .pick_file()
            {
                app.input_path = path.to_string_lossy().to_string();
                app.status = "入力パスを選択しました。".to_string();
            } else {
                app.status = "ファイル選択をキャンセルしました。".to_string();
            }
        }
        Message::LoadPressed => {
            let normalized = normalize_input_path(&app.input_path);
            let path = PathBuf::from(&normalized);
            if normalized.is_empty() || path.as_os_str().is_empty() {
                app.status = "エラー: 入力 JSON パスが空です。".to_string();
                return Task::none();
            }
            if !path.exists() {
                app.status = format!("エラー: ファイルが見つかりません: {}", path.display());
                return Task::none();
            }
            match fs::read_to_string(&path) {
                Ok(raw) => match serde_json::from_str::<Report>(&raw) {
                    Ok(r) => {
                        app.report = Some(r);
                        app.loaded_json_path = Some(path.clone());
                        app.status = format!("レポート JSON を読み込みました: {}", path.display());
                    }
                    Err(e) => {
                        app.status =
                            format!("エラー: JSON の解析に失敗しました: {}: {e}", path.display());
                    }
                },
                Err(e) => {
                    app.status = format!(
                        "エラー: ファイルを読めませんでした: {}: {e}",
                        path.display()
                    );
                }
            }
        }
        Message::OpenCashChart => {
            app.status = open_artifact(app, "cash_balance.png");
        }
        Message::OpenStockoutChart => {
            app.status = open_artifact(app, "daily_stockout.png");
        }
        Message::OpenSummary => {
            app.status = open_artifact(app, "summary.txt");
        }
        Message::GenerateReportArtifacts => {
            app.status = generate_report_artifacts(app);
        }
    }
    Task::none()
}

fn open_artifact(app: &App, filename: &str) -> String {
    let Some(json_path) = &app.loaded_json_path else {
        return "エラー: 先に JSON を読み込んでください。".to_string();
    };
    let Some(base_dir) = json_path.parent() else {
        return "エラー: JSON パスが不正です。".to_string();
    };
    let artifact = base_dir.join(filename);
    if !artifact.exists() {
        return format!("エラー: 成果物が見つかりません: {}", artifact.display());
    }
    match open_with_system(&artifact) {
        Ok(()) => format!("成果物を開きました: {}", artifact.display()),
        Err(e) => format!("エラー: 成果物を開けませんでした: {e}"),
    }
}

fn open_with_system(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &path.to_string_lossy()])
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    #[allow(unreachable_code)]
    Err("未対応の OS です".to_string())
}

fn generate_report_artifacts(app: &App) -> String {
    let Some(json_path) = &app.loaded_json_path else {
        return "エラー: 先に JSON を読み込んでください。".to_string();
    };
    let Some(out_dir) = json_path.parent() else {
        return "エラー: JSON パスが不正です。".to_string();
    };

    let ui_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(project_root) = ui_dir.parent() else {
        return "エラー: プロジェクトルートを解決できません。".to_string();
    };
    let reporting_python = project_root.join("reporting").join("python");
    if !reporting_python.exists() {
        return format!(
            "エラー: reporting/python が見つかりません: {}",
            reporting_python.display()
        );
    }

    let status = Command::new("uv")
        .current_dir(&reporting_python)
        .args([
            "run",
            "decision-report",
            "--input",
            &json_path.to_string_lossy(),
            "--out-dir",
            &out_dir.to_string_lossy(),
        ])
        .status();

    match status {
        Ok(s) if s.success() => format!("レポート成果物を生成しました: {}", out_dir.display()),
        Ok(s) => format!(
            "エラー: レポート生成が失敗しました。終了コード: {:?}",
            s.code()
        ),
        Err(e) => format!("エラー: uv の実行に失敗しました: {e}"),
    }
}

fn view(app: &App) -> Element<'_, Message> {
    let top_row = row![
        text_input("入力 JSON パス", &app.input_path)
            .on_input(Message::InputChanged)
            .padding(8)
            .width(Length::Fill),
        button("参照").on_press(Message::BrowsePressed),
        button("読込").on_press(Message::LoadPressed),
    ]
    .spacing(8);

    let (
        scenario_name,
        generated_at,
        min_cash,
        total_stockout,
        stockout_rate,
        alerts,
        cash_series,
        inventory_series,
    ) = if let Some(r) = &app.report {
        (
            r.scenario.name.clone(),
            r.generated_at.clone(),
            format!("{} 円", with_commas(r.kpi.min_cash)),
            r.kpi.total_stockout_qty.to_string(),
            format!("{:.2}%", r.kpi.stockout_rate * 100.0),
            r.alerts.clone(),
            r.cash_series.clone(),
            r.inventory_series.clone(),
        )
    } else {
        (
            "未読込".to_string(),
            "未読込".to_string(),
            "未読込".to_string(),
            "未読込".to_string(),
            "未読込".to_string(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    };

    let header = row![
        text(format!("シナリオ: {scenario_name}")),
        text(format!("生成時刻: {generated_at}")),
    ]
    .spacing(20);

    let kpi_row = row![
        kpi_card("最小資金残高", min_cash),
        kpi_card("総欠品数量", total_stockout),
        kpi_card("欠品率", stockout_rate),
    ]
    .spacing(10);

    let mut alert_col = column![text(format!("アラート ({})", alerts.len()))].spacing(6);
    if alerts.is_empty() {
        alert_col = alert_col.push(text("- （なし）"));
    } else {
        for a in alerts.iter().take(10) {
            let item = a.item_id.as_deref().unwrap_or("-");
            alert_col = alert_col.push(text(format!(
                "- [{}] {} {} ({}) {}",
                a.severity.to_uppercase(),
                a.date,
                a.code,
                item,
                a.message
            )));
        }
    }

    let artifacts = row![
        button("レポート成果物を生成").on_press(Message::GenerateReportArtifacts),
        button("cash_balance.png を開く").on_press(Message::OpenCashChart),
        button("daily_stockout.png を開く").on_press(Message::OpenStockoutChart),
        button("summary.txt を開く").on_press(Message::OpenSummary),
    ]
    .spacing(8);

    let metrics = summarize_metrics(&cash_series, &inventory_series);
    let metrics_box = container(
        column![
            text("計算済みサマリ"),
            text(format!(
                "- 純資金増減: {} 円",
                with_commas(metrics.net_cash_change)
            )),
            text(format!(
                "- 総入金 / 総出金: {} / {} 円",
                with_commas(metrics.total_inflow),
                with_commas(metrics.total_outflow)
            )),
            text(format!(
                "- 需要 / 販売 / 欠品: {} / {} / {}",
                metrics.total_demand, metrics.total_sold, metrics.total_stockout
            )),
            text(format!(
                "- 再計算した欠品率: {:.2}%",
                metrics.stockout_rate * 100.0
            )),
            text(format!(
                "- 欠品が最も多い品目: {} ({})",
                metrics.top_stockout_item, metrics.top_stockout_qty
            )),
        ]
        .spacing(4),
    )
    .padding(10);

    let cash_view = cash_series_view(&cash_series);
    let stockout_view = daily_stockout_view(&inventory_series);

    let content = column![
        top_row,
        header,
        kpi_row,
        metrics_box,
        text("資金推移"),
        scrollable(cash_view).height(Length::Fixed(180.0)),
        text("日次欠品"),
        scrollable(stockout_view)
            .height(Length::Fixed(160.0))
            .width(Length::Fill),
        scrollable(alert_col).height(Length::Fixed(160.0)),
        artifacts,
        text(format!("状態: {}", status_text(app))),
    ]
    .spacing(12)
    .padding(16);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn status_text(app: &App) -> &str {
    if app.status.is_empty() {
        "準備完了"
    } else {
        &app.status
    }
}

fn kpi_card(title: &'static str, value: String) -> Element<'static, Message> {
    container(column![text(title), text(value)].spacing(4))
        .padding(10)
        .width(Length::FillPortion(1))
        .into()
}

fn with_commas(v: i64) -> String {
    let s = v.abs().to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    let mut out: String = out.chars().rev().collect();
    if v < 0 {
        out.insert(0, '-');
    }
    out
}

fn normalize_input_path(raw: &str) -> String {
    let mut s = raw.trim().to_string();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s = s[1..s.len() - 1].to_string();
    }
    if let Some(stripped) = s.strip_prefix("file://") {
        s = stripped.to_string();
    }
    s
}

#[derive(Debug, Clone)]
struct ComputedMetrics {
    net_cash_change: i64,
    total_inflow: i64,
    total_outflow: i64,
    total_demand: u32,
    total_sold: u32,
    total_stockout: u32,
    stockout_rate: f64,
    top_stockout_item: String,
    top_stockout_qty: u32,
}

fn summarize_metrics(cash: &[CashPoint], inv: &[InventoryPoint]) -> ComputedMetrics {
    let total_inflow = cash.iter().map(|c| c.inflow).sum::<i64>();
    let total_outflow = cash.iter().map(|c| c.outflow).sum::<i64>();
    let net_cash_change = if let (Some(first), Some(last)) = (cash.first(), cash.last()) {
        last.cash - first.cash
    } else {
        0
    };

    let total_demand = inv.iter().map(|v| v.demand).sum::<u32>();
    let total_sold = inv.iter().map(|v| v.sold).sum::<u32>();
    let total_stockout = inv.iter().map(|v| v.stockout).sum::<u32>();
    let stockout_rate = if total_demand == 0 {
        0.0
    } else {
        total_stockout as f64 / total_demand as f64
    };

    let mut by_item: BTreeMap<String, u32> = BTreeMap::new();
    for row in inv {
        *by_item.entry(row.item_id.clone()).or_insert(0) += row.stockout;
    }
    let (top_stockout_item, top_stockout_qty) = by_item
        .into_iter()
        .max_by_key(|(_, qty)| *qty)
        .unwrap_or_else(|| ("-".to_string(), 0));

    ComputedMetrics {
        net_cash_change,
        total_inflow,
        total_outflow,
        total_demand,
        total_sold,
        total_stockout,
        stockout_rate,
        top_stockout_item,
        top_stockout_qty,
    }
}

fn cash_series_view(rows: &[CashPoint]) -> iced::widget::Column<'static, Message> {
    if rows.is_empty() {
        return column![text("- （資金推移なし）")];
    }
    let min = rows.iter().map(|r| r.cash).min().unwrap_or(0) as f32;
    let max = rows.iter().map(|r| r.cash).max().unwrap_or(0) as f32;
    let range = (max - min).max(1.0);

    let mut col = column![].spacing(6);
    for r in rows {
        let ratio = ((r.cash as f32 - min) / range).clamp(0.0, 1.0);
        col = col.push(
            row![
                text(format!(
                    "{}  残高={}  入出金={}/{}",
                    r.date,
                    with_commas(r.cash),
                    with_commas(r.inflow),
                    with_commas(r.outflow)
                ))
                .width(Length::FillPortion(3)),
                progress_bar(0.0..=1.0, ratio).width(Length::FillPortion(2)),
            ]
            .spacing(10),
        );
    }
    col
}

fn daily_stockout_view(rows: &[InventoryPoint]) -> iced::widget::Column<'static, Message> {
    if rows.is_empty() {
        return column![text("- （在庫推移なし）").width(Length::Fill)];
    }
    let mut by_day: BTreeMap<String, u32> = BTreeMap::new();
    for r in rows {
        *by_day.entry(r.date.clone()).or_insert(0) += r.stockout;
    }
    let mut col = column![].spacing(6).width(Length::Fill);
    for (date, qty) in by_day {
        col = col.push(
            row![
                text(date).width(Length::FillPortion(2)),
                text(format!("欠品={qty}")).width(Length::FillPortion(3)),
            ]
            .spacing(10)
            .width(Length::Fill),
        );
    }
    col
}
