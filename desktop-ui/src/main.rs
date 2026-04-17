use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length, Task, Theme};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::BTreeMap;

fn main() -> iced::Result {
    iced::application("Decision Pack UI", update, view)
        .theme(|_| Theme::TokyoNight)
        .run_with(|| {
            let app = App::default();
            let base = app.api_base_url.clone();
            (
                app,
                Task::batch(vec![
                    load_customers_task(base.clone()),
                    load_items_task(base.clone(), String::new()),
                    load_simulations_task(base),
                ]),
            )
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Tab {
    #[default]
    Customers,
    Inventory,
    Simulations,
}

#[derive(Debug)]
struct App {
    api_base_url: String,
    status: String,
    active_tab: Tab,
    customer_query: String,
    customers: Vec<CustomerSummary>,
    selected_customer_id: Option<String>,
    customer_detail: Option<CustomerDetail>,
    customer_purchases: Vec<CustomerPurchase>,
    customer_next_buy: Vec<CustomerNextBuy>,
    item_query: String,
    items: Vec<ItemSummary>,
    selected_item_id: Option<String>,
    item_detail: Option<ItemDetail>,
    item_inventory: Option<ItemInventory>,
    item_risk: Option<ItemRisk>,
    simulations: Vec<SimulationSummary>,
    selected_run_id: Option<String>,
    simulation_detail: Option<SimulationDetail>,
    simulation_report: Option<SimulationReport>,
    is_loading: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            api_base_url: "http://127.0.0.1:8080".to_string(),
            status: "API から初期データを読み込みます。".to_string(),
            active_tab: Tab::Customers,
            customer_query: String::new(),
            customers: Vec::new(),
            selected_customer_id: None,
            customer_detail: None,
            customer_purchases: Vec::new(),
            customer_next_buy: Vec::new(),
            item_query: String::new(),
            items: Vec::new(),
            selected_item_id: None,
            item_detail: None,
            item_inventory: None,
            item_risk: None,
            simulations: Vec::new(),
            selected_run_id: None,
            simulation_detail: None,
            simulation_report: None,
            is_loading: false,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    ApiBaseUrlChanged(String),
    SwitchTab(Tab),
    CustomerQueryChanged(String),
    RefreshCustomers,
    CustomersLoaded(Result<Vec<CustomerSummary>, String>),
    SelectCustomer(String),
    CustomerBundleLoaded(Result<CustomerBundle, String>),
    ItemQueryChanged(String),
    RefreshItems,
    ItemsLoaded(Result<Vec<ItemSummary>, String>),
    SelectItem(String),
    ItemBundleLoaded(Result<ItemBundle, String>),
    RefreshSimulations,
    SimulationsLoaded(Result<Vec<SimulationSummary>, String>),
    SelectSimulation(String),
    SimulationBundleLoaded(Result<SimulationBundle, String>),
    RunSimulation,
    SimulationCreated(Result<SimulationDetail, String>),
}

#[derive(Debug, Clone, Deserialize)]
struct CustomerSummary {
    customer_id: String,
    full_name: String,
    email: Option<String>,
    status: Option<String>,
    tier: Option<String>,
    country: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CustomerDetail {
    customer_id: String,
    full_name: String,
    email: Option<String>,
    phone: Option<String>,
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
    status: Option<String>,
    tier: Option<String>,
    preferred_language: Option<String>,
    marketing_opt_in: Option<bool>,
    total_spend: Option<f64>,
    order_count: Option<i32>,
    last_purchase_date: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CustomerPurchase {
    order_id: String,
    ordered_at: String,
    order_status: String,
    item_id: String,
    item_name: String,
    quantity: i32,
    unit_price: Option<f64>,
    line_amount: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct CustomerNextBuy {
    item_id: String,
    item_name: String,
    score: f64,
    rank: i32,
    as_of: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ItemSummary {
    item_id: String,
    item_name: String,
    category: String,
    is_active: bool,
    on_hand: Option<i32>,
    on_order: Option<i32>,
    reserved_qty: Option<i32>,
    updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ItemDetail {
    item_id: String,
    item_name: String,
    category: String,
    uom: Option<String>,
    is_active: bool,
    lead_time_days: i32,
    moq: Option<i32>,
    lot_size: Option<i32>,
    updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ItemInventory {
    item_id: String,
    on_hand: i32,
    on_order: i32,
    reserved_qty: i32,
    updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ItemRisk {
    run_id: String,
    scenario_id: String,
    scenario_name: String,
    risk_level: Option<String>,
    recommended_reorder_qty: Option<i32>,
    expected_stockout_qty: Option<i32>,
    expected_days_on_hand: Option<f64>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SimulationSummary {
    run_id: String,
    scenario_id: String,
    scenario_name: String,
    status: String,
    requested_at: String,
    completed_at: Option<String>,
    report_schema_version: Option<String>,
    report_uri: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct SimulationDetail {
    run_id: String,
    scenario_id: String,
    scenario_name: String,
    status: String,
    requested_at: String,
    started_at: Option<String>,
    completed_at: Option<String>,
    report_schema_version: Option<String>,
    report_uri: Option<String>,
    report_available: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct SimulationReportEnvelope {
    #[allow(dead_code)]
    run_id: String,
    report: SimulationReport,
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
    days_on_hand_avg: f64,
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
struct SimulationReport {
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
    on_hand: u32,
    on_order: u32,
    demand: u32,
    sold: u32,
    stockout: u32,
}

#[derive(Debug, Clone)]
struct CustomerBundle {
    detail: CustomerDetail,
    purchases: Vec<CustomerPurchase>,
    next_buy: Vec<CustomerNextBuy>,
}

#[derive(Debug, Clone)]
struct ItemBundle {
    detail: ItemDetail,
    inventory: ItemInventory,
    risk: Option<ItemRisk>,
}

#[derive(Debug, Clone)]
struct SimulationBundle {
    detail: SimulationDetail,
    report: Option<SimulationReport>,
}

#[derive(Debug, Serialize)]
struct CreateSimulationRequest {
    scenario_id: String,
    scenario_name: String,
    scenario_description: String,
    horizon_days: u32,
    initial_cash: i64,
    currency: String,
}

fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::ApiBaseUrlChanged(value) => {
            app.api_base_url = value;
            Task::none()
        }
        Message::SwitchTab(tab) => {
            app.active_tab = tab;
            Task::none()
        }
        Message::CustomerQueryChanged(value) => {
            app.customer_query = value;
            Task::none()
        }
        Message::RefreshCustomers => {
            app.is_loading = true;
            app.status = "顧客一覧を読み込みます。".to_string();
            load_customers_task(app.api_base_url.clone())
        }
        Message::CustomersLoaded(result) => {
            app.is_loading = false;
            match result {
                Ok(customers) => {
                    app.status = format!("顧客一覧を {} 件読み込みました。", customers.len());
                    app.customers = customers;
                }
                Err(error) => app.status = error,
            }
            Task::none()
        }
        Message::SelectCustomer(customer_id) => {
            app.selected_customer_id = Some(customer_id.clone());
            app.is_loading = true;
            app.status = format!("顧客 {} の詳細を読み込みます。", customer_id);
            load_customer_bundle_task(app.api_base_url.clone(), customer_id)
        }
        Message::CustomerBundleLoaded(result) => {
            app.is_loading = false;
            match result {
                Ok(bundle) => {
                    app.status =
                        format!("顧客 {} の詳細を更新しました。", bundle.detail.customer_id);
                    app.customer_detail = Some(bundle.detail);
                    app.customer_purchases = bundle.purchases;
                    app.customer_next_buy = bundle.next_buy;
                }
                Err(error) => app.status = error,
            }
            Task::none()
        }
        Message::ItemQueryChanged(value) => {
            app.item_query = value;
            Task::none()
        }
        Message::RefreshItems => {
            app.is_loading = true;
            app.status = "在庫一覧を読み込みます。".to_string();
            load_items_task(app.api_base_url.clone(), app.item_query.clone())
        }
        Message::ItemsLoaded(result) => {
            app.is_loading = false;
            match result {
                Ok(items) => {
                    app.status = format!("品目一覧を {} 件読み込みました。", items.len());
                    app.items = items;
                }
                Err(error) => app.status = error,
            }
            Task::none()
        }
        Message::SelectItem(item_id) => {
            app.selected_item_id = Some(item_id.clone());
            app.is_loading = true;
            app.status = format!("品目 {} の詳細を読み込みます。", item_id);
            load_item_bundle_task(app.api_base_url.clone(), item_id)
        }
        Message::ItemBundleLoaded(result) => {
            app.is_loading = false;
            match result {
                Ok(bundle) => {
                    app.status = format!("品目 {} の詳細を更新しました。", bundle.detail.item_id);
                    app.item_detail = Some(bundle.detail);
                    app.item_inventory = Some(bundle.inventory);
                    app.item_risk = bundle.risk;
                }
                Err(error) => app.status = error,
            }
            Task::none()
        }
        Message::RefreshSimulations => {
            app.is_loading = true;
            app.status = "シミュレーション一覧を読み込みます。".to_string();
            load_simulations_task(app.api_base_url.clone())
        }
        Message::SimulationsLoaded(result) => {
            app.is_loading = false;
            match result {
                Ok(simulations) => {
                    app.status = format!(
                        "シミュレーション一覧を {} 件読み込みました。",
                        simulations.len()
                    );
                    app.simulations = simulations;
                }
                Err(error) => app.status = error,
            }
            Task::none()
        }
        Message::SelectSimulation(run_id) => {
            app.selected_run_id = Some(run_id.clone());
            app.is_loading = true;
            app.status = format!("シミュレーション {} を読み込みます。", run_id);
            load_simulation_bundle_task(app.api_base_url.clone(), run_id)
        }
        Message::SimulationBundleLoaded(result) => {
            app.is_loading = false;
            match result {
                Ok(bundle) => {
                    app.status =
                        format!("シミュレーション {} を更新しました。", bundle.detail.run_id);
                    app.simulation_detail = Some(bundle.detail);
                    app.simulation_report = bundle.report;
                }
                Err(error) => app.status = error,
            }
            Task::none()
        }
        Message::RunSimulation => {
            app.is_loading = true;
            app.status = "シミュレーションを起動します。".to_string();
            create_simulation_task(app.api_base_url.clone())
        }
        Message::SimulationCreated(result) => {
            app.is_loading = false;
            match result {
                Ok(detail) => {
                    let run_id = detail.run_id.clone();
                    app.status = format!("シミュレーション {} を実行しました。", run_id);
                    app.selected_run_id = Some(run_id.clone());
                    return Task::batch(vec![
                        load_simulations_task(app.api_base_url.clone()),
                        load_simulation_bundle_task(app.api_base_url.clone(), run_id),
                    ]);
                }
                Err(error) => app.status = error,
            }
            Task::none()
        }
    }
}

fn view(app: &App) -> Element<'_, Message> {
    let controls = row![
        text("API"),
        text_input("http://127.0.0.1:8080", &app.api_base_url)
            .on_input(Message::ApiBaseUrlChanged)
            .padding(8)
            .width(Length::FillPortion(3)),
        button("顧客再読込").on_press(Message::RefreshCustomers),
        button("在庫再読込").on_press(Message::RefreshItems),
        button("シミュレーション再読込").on_press(Message::RefreshSimulations),
    ]
    .spacing(8);

    let tabs = row![
        tab_button("顧客", Tab::Customers, app.active_tab),
        tab_button("在庫", Tab::Inventory, app.active_tab),
        tab_button("シミュレーション", Tab::Simulations, app.active_tab),
    ]
    .spacing(8);

    let status = if app.is_loading {
        format!("状態: {}（処理中）", app.status)
    } else {
        format!("状態: {}", app.status)
    };

    let content = match app.active_tab {
        Tab::Customers => customers_view(app),
        Tab::Inventory => inventory_view(app),
        Tab::Simulations => simulations_view(app),
    };

    container(
        column![controls, tabs, content, text(status)]
            .spacing(12)
            .padding(16),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn customers_view(app: &App) -> Element<'_, Message> {
    let filter = app.customer_query.trim().to_lowercase();
    let customers = app
        .customers
        .iter()
        .filter(|customer| {
            filter.is_empty()
                || customer.customer_id.to_lowercase().contains(&filter)
                || customer.full_name.to_lowercase().contains(&filter)
                || customer
                    .email
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains(&filter)
        })
        .collect::<Vec<_>>();

    let mut list = column![
        text_input("顧客フィルタ", &app.customer_query)
            .on_input(Message::CustomerQueryChanged)
            .padding(8)
    ]
    .spacing(8);
    for customer in customers {
        let label = format!(
            "{} | {} | {} | {} | {}",
            customer.customer_id,
            customer.full_name,
            customer.status.as_deref().unwrap_or("-"),
            customer.tier.as_deref().unwrap_or("-"),
            customer.country.as_deref().unwrap_or("-")
        );
        list = list.push(
            button(text(label))
                .width(Length::Fill)
                .on_press(Message::SelectCustomer(customer.customer_id.clone())),
        );
    }

    let detail = customer_detail_panel(app);
    row![
        container(scrollable(list)).width(Length::FillPortion(2)),
        container(detail).width(Length::FillPortion(3)),
    ]
    .spacing(12)
    .into()
}

fn customer_detail_panel(app: &App) -> Element<'_, Message> {
    let Some(detail) = &app.customer_detail else {
        return container(text("顧客を選択してください。"))
            .width(Length::Fill)
            .into();
    };

    let mut purchases = column![text("購入履歴")].spacing(4);
    if app.customer_purchases.is_empty() {
        purchases = purchases.push(text("- 購入履歴なし"));
    } else {
        for row in app.customer_purchases.iter().take(12) {
            purchases = purchases.push(text(format!(
                "{} | {} | {} ({}) x{} | unit={} | line={} | {}",
                row.ordered_at,
                row.order_id,
                row.item_name,
                row.item_id,
                row.quantity,
                money_opt(row.unit_price),
                money_opt(row.line_amount),
                row.order_status
            )));
        }
    }

    let mut next_buy = column![text("次回購入候補")].spacing(4);
    if app.customer_next_buy.is_empty() {
        next_buy = next_buy.push(text("- 候補なし"));
    } else {
        for row in app.customer_next_buy.iter().take(10) {
            next_buy = next_buy.push(text(format!(
                "#{} {} ({}) score={:.3} as_of={}",
                row.rank, row.item_name, row.item_id, row.score, row.as_of
            )));
        }
    }

    scrollable(
        column![
            text(format!("{} / {}", detail.customer_id, detail.full_name)),
            text(format!(
                "状態={} | tier={} | 国={}",
                detail.status.as_deref().unwrap_or("-"),
                detail.tier.as_deref().unwrap_or("-"),
                detail.country.as_deref().unwrap_or("-")
            )),
            text(format!(
                "言語={} | 連絡先={} | マーケ可={}",
                detail.preferred_language.as_deref().unwrap_or("-"),
                detail.email.as_deref().unwrap_or("-"),
                yes_no(detail.marketing_opt_in)
            )),
            text(format!(
                "累計売上={} | 注文回数={} | 最終購入={}",
                money_opt(detail.total_spend),
                detail.order_count.unwrap_or_default(),
                detail.last_purchase_date.as_deref().unwrap_or("-")
            )),
            text(format!(
                "地域={} / {} | 電話={}",
                detail.region.as_deref().unwrap_or("-"),
                detail.city.as_deref().unwrap_or("-"),
                detail.phone.as_deref().unwrap_or("-")
            )),
            text(format!("備考={}", detail.notes.as_deref().unwrap_or("-"))),
            purchases,
            next_buy,
        ]
        .spacing(8),
    )
    .into()
}

fn inventory_view(app: &App) -> Element<'_, Message> {
    let list_header = row![
        text_input("品目検索", &app.item_query)
            .on_input(Message::ItemQueryChanged)
            .padding(8)
            .width(Length::Fill),
        button("検索").on_press(Message::RefreshItems),
    ]
    .spacing(8);

    let mut list = column![list_header].spacing(8);
    for item in &app.items {
        list = list.push(
            button(text(format!(
                "{} | {} | {} | active={} | on_hand={} | on_order={} | reserved={} | {}",
                item.item_id,
                item.item_name,
                item.category,
                yes_no(Some(item.is_active)),
                item.on_hand.unwrap_or_default(),
                item.on_order.unwrap_or_default(),
                item.reserved_qty.unwrap_or_default(),
                item.updated_at.as_deref().unwrap_or("-")
            )))
            .width(Length::Fill)
            .on_press(Message::SelectItem(item.item_id.clone())),
        );
    }

    let detail = item_detail_panel(app);
    row![
        container(scrollable(list)).width(Length::FillPortion(2)),
        container(detail).width(Length::FillPortion(3)),
    ]
    .spacing(12)
    .into()
}

fn item_detail_panel(app: &App) -> Element<'_, Message> {
    let Some(detail) = &app.item_detail else {
        return container(text("品目を選択してください。"))
            .width(Length::Fill)
            .into();
    };
    let inventory = app.item_inventory.as_ref();
    let risk = app.item_risk.as_ref();

    container(
        column![
            text(format!("{} / {}", detail.item_id, detail.item_name)),
            text(format!(
                "カテゴリ={} | active={} | UOM={}",
                detail.category,
                yes_no(Some(detail.is_active)),
                detail.uom.as_deref().unwrap_or("-")
            )),
            text(format!(
                "lead_time={}日 | moq={} | lot_size={}",
                detail.lead_time_days,
                detail.moq.unwrap_or_default(),
                detail.lot_size.unwrap_or_default()
            )),
            text(format!(
                "在庫={} | 発注残={} | 引当={} | 更新={} | inventory_item={}",
                inventory.map(|row| row.on_hand).unwrap_or_default(),
                inventory.map(|row| row.on_order).unwrap_or_default(),
                inventory.map(|row| row.reserved_qty).unwrap_or_default(),
                inventory
                    .map(|row| row.updated_at.as_str())
                    .unwrap_or(detail.updated_at.as_str()),
                inventory
                    .map(|row| row.item_id.as_str())
                    .unwrap_or(detail.item_id.as_str())
            )),
            text(format!(
                "最新リスク={} | 推奨補充={} | 想定欠品={} | 平均在庫日数={}",
                risk.and_then(|row| row.risk_level.as_deref())
                    .unwrap_or("未計算"),
                risk.and_then(|row| row.recommended_reorder_qty)
                    .unwrap_or_default(),
                risk.and_then(|row| row.expected_stockout_qty)
                    .unwrap_or_default(),
                risk.and_then(|row| row.expected_days_on_hand)
                    .map(|value| format!("{value:.1}"))
                    .unwrap_or_else(|| "-".to_string())
            )),
            if let Some(risk) = risk {
                text(format!(
                    "参照 run={} / scenario={} ({}) / requested_at={}",
                    risk.run_id, risk.scenario_name, risk.scenario_id, risk.requested_at
                ))
            } else {
                text("シミュレーション結果はまだありません。")
            },
        ]
        .spacing(8),
    )
    .into()
}

fn simulations_view(app: &App) -> Element<'_, Message> {
    let controls = row![
        button("ベースライン実行").on_press(Message::RunSimulation),
        button("一覧更新").on_press(Message::RefreshSimulations),
    ]
    .spacing(8);

    let mut list = column![controls].spacing(8);
    for simulation in &app.simulations {
        list = list.push(
            button(text(format!(
                "{} | {} | {} | {} | {} | {} | {} | {}",
                simulation.run_id,
                simulation.scenario_name,
                simulation.scenario_id,
                simulation.status,
                simulation.requested_at,
                simulation.completed_at.as_deref().unwrap_or("-"),
                simulation.report_schema_version.as_deref().unwrap_or("-"),
                simulation.report_uri.as_deref().unwrap_or("-")
            )))
            .width(Length::Fill)
            .on_press(Message::SelectSimulation(simulation.run_id.clone())),
        );
    }

    let detail = simulation_detail_panel(app);
    row![
        container(scrollable(list)).width(Length::FillPortion(2)),
        container(detail).width(Length::FillPortion(3)),
    ]
    .spacing(12)
    .into()
}

fn simulation_detail_panel(app: &App) -> Element<'_, Message> {
    let Some(detail) = &app.simulation_detail else {
        return container(text("シミュレーションを選択してください。"))
            .width(Length::Fill)
            .into();
    };

    let report_summary = if let Some(report) = &app.simulation_report {
        let top_stockout = top_stockout_items(&report.inventory_series);
        column![
            text(format!(
                "schema={} | generated_at={}",
                report.schema_version.as_deref().unwrap_or("-"),
                report.generated_at
            )),
            text(format!(
                "cash期間={}..{} | total_inflow={} | total_outflow={} | final_cash={}",
                report
                    .cash_series
                    .first()
                    .map(|row| row.date.as_str())
                    .unwrap_or("-"),
                report
                    .cash_series
                    .last()
                    .map(|row| row.date.as_str())
                    .unwrap_or("-"),
                with_commas(report.cash_series.iter().map(|row| row.inflow).sum::<i64>()),
                with_commas(report.cash_series.iter().map(|row| row.outflow).sum::<i64>()),
                with_commas(report.cash_series.last().map(|row| row.cash).unwrap_or_default())
            )),
            text(format!(
                "scenario={} | min_cash={} | total_stockout={} | stockout_rate={:.2}% | avg_days_on_hand={:.2}",
                report.scenario.name,
                with_commas(report.kpi.min_cash),
                report.kpi.total_stockout_qty,
                report.kpi.stockout_rate * 100.0,
                report.kpi.days_on_hand_avg
            )),
            text(format!(
                "cash points={} | inventory points={}",
                report.cash_series.len(),
                report.inventory_series.len()
            )),
            text(format!(
                "inventory期間={}..{} | total_demand={} | total_sold={}",
                report
                    .inventory_series
                    .first()
                    .map(|row| row.date.as_str())
                    .unwrap_or("-"),
                report
                    .inventory_series
                    .last()
                    .map(|row| row.date.as_str())
                    .unwrap_or("-"),
                report.inventory_series.iter().map(|row| row.demand).sum::<u32>(),
                report.inventory_series.iter().map(|row| row.sold).sum::<u32>()
            )),
            text("主要アラート"),
            alerts_view(&report.alerts),
            text("欠品上位品目"),
            top_stockout,
        ]
        .spacing(6)
    } else {
        column![text("レポート JSON はまだありません。")].spacing(6)
    };

    scrollable(
        column![
            text(format!("run={} / {}", detail.run_id, detail.scenario_name)),
            text(format!(
                "status={} | requested_at={} | report_available={}",
                detail.status, detail.requested_at, detail.report_available
            )),
            text(format!(
                "started_at={} | completed_at={}",
                detail.started_at.as_deref().unwrap_or("-"),
                detail.completed_at.as_deref().unwrap_or("-")
            )),
            text(format!(
                "schema={} | report_uri={} | scenario_id={}",
                detail.report_schema_version.as_deref().unwrap_or("-"),
                detail.report_uri.as_deref().unwrap_or("-"),
                detail.scenario_id
            )),
            report_summary,
        ]
        .spacing(8),
    )
    .into()
}

fn alerts_view(alerts: &[Alert]) -> iced::widget::Column<'_, Message> {
    let mut col = column![].spacing(4);
    if alerts.is_empty() {
        return col.push(text("- アラートなし"));
    }
    for alert in alerts.iter().take(8) {
        col = col.push(text(format!(
            "{} | {} | {} | {} | {}",
            alert.date,
            alert.severity,
            alert.code,
            alert.item_id.as_deref().unwrap_or("-"),
            alert.message
        )));
    }
    col
}

fn top_stockout_items(rows: &[InventoryPoint]) -> iced::widget::Column<'_, Message> {
    let mut by_item: BTreeMap<String, (u32, u32, u32)> = BTreeMap::new();
    for row in rows {
        let entry = by_item.entry(row.item_id.clone()).or_insert((0, 0, 0));
        entry.0 += row.stockout;
        entry.1 += row.on_hand;
        entry.2 += row.on_order;
    }

    let mut ranked = by_item.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|a, b| b.1.0.cmp(&a.1.0).then(a.0.cmp(&b.0)));

    let mut col = column![].spacing(4);
    if ranked.is_empty() {
        return col.push(text("- データなし"));
    }
    for (item_id, (stockout, on_hand, on_order)) in ranked.into_iter().take(8) {
        col = col.push(text(format!(
            "{} | stockout={} | on_hand_sum={} | on_order_sum={}",
            item_id, stockout, on_hand, on_order
        )));
    }
    col
}

fn tab_button<'a>(label: &'a str, tab: Tab, active: Tab) -> Element<'a, Message> {
    let caption = if tab == active {
        format!("[{}]", label)
    } else {
        label.to_string()
    };
    button(text(caption))
        .on_press(Message::SwitchTab(tab))
        .into()
}

fn load_customers_task(base: String) -> Task<Message> {
    Task::perform(
        async move { fetch_customers(&base) },
        Message::CustomersLoaded,
    )
}

fn load_customer_bundle_task(base: String, customer_id: String) -> Task<Message> {
    Task::perform(
        async move { fetch_customer_bundle(&base, &customer_id) },
        Message::CustomerBundleLoaded,
    )
}

fn load_items_task(base: String, query: String) -> Task<Message> {
    Task::perform(
        async move { fetch_items(&base, &query) },
        Message::ItemsLoaded,
    )
}

fn load_item_bundle_task(base: String, item_id: String) -> Task<Message> {
    Task::perform(
        async move { fetch_item_bundle(&base, &item_id) },
        Message::ItemBundleLoaded,
    )
}

fn load_simulations_task(base: String) -> Task<Message> {
    Task::perform(
        async move { fetch_simulations(&base) },
        Message::SimulationsLoaded,
    )
}

fn load_simulation_bundle_task(base: String, run_id: String) -> Task<Message> {
    Task::perform(
        async move { fetch_simulation_bundle(&base, &run_id) },
        Message::SimulationBundleLoaded,
    )
}

fn create_simulation_task(base: String) -> Task<Message> {
    Task::perform(
        async move { create_simulation(&base) },
        Message::SimulationCreated,
    )
}

fn fetch_customers(base: &str) -> Result<Vec<CustomerSummary>, String> {
    get_json(&format!(
        "{}/api/v1/customers?limit=100",
        normalize_base(base)
    ))
}

fn fetch_customer_bundle(base: &str, customer_id: &str) -> Result<CustomerBundle, String> {
    let base = normalize_base(base);
    Ok(CustomerBundle {
        detail: get_json(&format!("{base}/api/v1/customers/{customer_id}"))?,
        purchases: get_json(&format!(
            "{base}/api/v1/customers/{customer_id}/purchases?limit=50"
        ))?,
        next_buy: get_json(&format!(
            "{base}/api/v1/customers/{customer_id}/next-buy?limit=20"
        ))?,
    })
}

fn fetch_items(base: &str, query: &str) -> Result<Vec<ItemSummary>, String> {
    let url = if query.trim().is_empty() {
        format!("{}/api/v1/items?limit=100", normalize_base(base))
    } else {
        format!(
            "{}/api/v1/items?limit=100&q={}",
            normalize_base(base),
            urlencoding::encode(query.trim())
        )
    };
    get_json(&url)
}

fn fetch_item_bundle(base: &str, item_id: &str) -> Result<ItemBundle, String> {
    let base = normalize_base(base);
    Ok(ItemBundle {
        detail: get_json(&format!("{base}/api/v1/items/{item_id}"))?,
        inventory: get_json(&format!("{base}/api/v1/items/{item_id}/inventory"))?,
        risk: get_json_optional(&format!("{base}/api/v1/items/{item_id}/risk"))?,
    })
}

fn fetch_simulations(base: &str) -> Result<Vec<SimulationSummary>, String> {
    get_json(&format!(
        "{}/api/v1/simulations?limit=50",
        normalize_base(base)
    ))
}

fn fetch_simulation_bundle(base: &str, run_id: &str) -> Result<SimulationBundle, String> {
    let base = normalize_base(base);
    let detail: SimulationDetail = get_json(&format!("{base}/api/v1/simulations/{run_id}"))?;
    let report = get_json_optional::<SimulationReportEnvelope>(&format!(
        "{base}/api/v1/simulations/{run_id}/report"
    ))?
    .map(|envelope| envelope.report);
    Ok(SimulationBundle { detail, report })
}

fn create_simulation(base: &str) -> Result<SimulationDetail, String> {
    post_json(
        &format!("{}/api/v1/simulations", normalize_base(base)),
        &CreateSimulationRequest {
            scenario_id: "baseline-api".to_string(),
            scenario_name: "ベースライン API 実行".to_string(),
            scenario_description: "GUI から起動した既定シナリオ".to_string(),
            horizon_days: 30,
            initial_cash: 1_000_000,
            currency: "JPY".to_string(),
        },
    )
}

fn get_json<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    let client = Client::new();
    let response = client.get(url).send().map_err(|error| error.to_string())?;
    parse_response(response)
}

fn get_json_optional<T: DeserializeOwned>(url: &str) -> Result<Option<T>, String> {
    let client = Client::new();
    let response = client.get(url).send().map_err(|error| error.to_string())?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    parse_response(response).map(Some)
}

fn post_json<TReq: Serialize, TRes: DeserializeOwned>(
    url: &str,
    payload: &TReq,
) -> Result<TRes, String> {
    let client = Client::new();
    let response = client
        .post(url)
        .json(payload)
        .send()
        .map_err(|error| error.to_string())?;
    parse_response(response)
}

fn parse_response<T: DeserializeOwned>(response: reqwest::blocking::Response) -> Result<T, String> {
    if response.status().is_success() {
        response.json::<T>().map_err(|error| error.to_string())
    } else {
        let status = response.status();
        let body = response.text().unwrap_or_else(|_| String::new());
        Err(format!("API error {}: {}", status, body))
    }
}

fn normalize_base(base: &str) -> String {
    base.trim_end_matches('/').to_string()
}

fn with_commas(value: i64) -> String {
    let s = value.abs().to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (idx, ch) in s.chars().rev().enumerate() {
        if idx > 0 && idx % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    let mut out: String = out.chars().rev().collect();
    if value < 0 {
        out.insert(0, '-');
    }
    out
}

fn money_opt(value: Option<f64>) -> String {
    value
        .map(|amount| format!("{amount:.2}"))
        .unwrap_or_else(|| "-".to_string())
}

fn yes_no(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "yes",
        Some(false) => "no",
        None => "-",
    }
}
