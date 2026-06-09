use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Background, Border, Color, Degrees, Element, Font, Length, Shadow, Task, Theme, Vector};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const DEFAULT_UI_FONT: Font = Font::with_name("Meiryo UI");
const HAN_UI_FONT: Font = Font::with_name("Microsoft YaHei UI");
const DEVANAGARI_UI_FONT: Font = Font::with_name("Nirmala UI");
const TEXT_PRIMARY: Color = Color::from_rgb(0.93, 0.95, 1.0);
const TEXT_MUTED: Color = Color::from_rgb(0.63, 0.68, 0.84);
const PANEL_BG: Color = Color::from_rgba(0.055, 0.065, 0.14, 0.84);
const BORDER_SUBTLE: Color = Color::from_rgba(0.55, 0.60, 1.0, 0.24);
const ACCENT: Color = Color::from_rgb(0.58, 0.38, 1.0);
const ACCENT_BRIGHT: Color = Color::from_rgb(0.25, 0.86, 1.0);

fn main() -> iced::Result {
    let mut application = iced::application("Decision Pack UI", update, view)
        .theme(|_| Theme::TokyoNight)
        .default_font(DEFAULT_UI_FONT);

    for font_bytes in load_ui_font_bytes() {
        application = application.font(font_bytes);
    }

    application.run_with(|| {
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

fn load_ui_font_bytes() -> Vec<Vec<u8>> {
    candidate_font_paths()
        .into_iter()
        .filter_map(|path| fs::read(path).ok())
        .collect()
}

fn candidate_font_paths() -> Vec<PathBuf> {
    let windows_dir = std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    let fonts_dir = windows_dir.join("Fonts");

    ["meiryo.ttc", "NotoSansJP-VF.ttf", "msyh.ttc", "Nirmala.ttc"]
        .into_iter()
        .map(|name| fonts_dir.join(name))
        .collect()
}

fn font_for_content(content: &str) -> Font {
    if content.chars().any(is_devanagari_char) {
        DEVANAGARI_UI_FONT
    } else if content.chars().any(is_han_char) && !content.chars().any(is_japanese_kana) {
        HAN_UI_FONT
    } else {
        DEFAULT_UI_FONT
    }
}

fn is_devanagari_char(ch: char) -> bool {
    let code = ch as u32;
    (0x0900..=0x097f).contains(&code)
        || (0x1cd0..=0x1cff).contains(&code)
        || (0xa8e0..=0xa8ff).contains(&code)
}

fn is_han_char(ch: char) -> bool {
    let code = ch as u32;
    (0x3400..=0x4dbf).contains(&code)
        || (0x4e00..=0x9fff).contains(&code)
        || (0xf900..=0xfaff).contains(&code)
}

fn is_japanese_kana(ch: char) -> bool {
    let code = ch as u32;
    (0x3040..=0x309f).contains(&code)
        || (0x30a0..=0x30ff).contains(&code)
        || (0x31f0..=0x31ff).contains(&code)
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
        label_text("API"),
        styled_text_input("http://127.0.0.1:8080", &app.api_base_url)
            .on_input(Message::ApiBaseUrlChanged)
            .padding(8)
            .width(Length::FillPortion(3)),
        action_button("顧客再読込").on_press(Message::RefreshCustomers),
        action_button("在庫再読込").on_press(Message::RefreshItems),
        action_button("シミュレーション再読込").on_press(Message::RefreshSimulations),
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

    let shell = column![
        text("Decision Pack")
            .font(DEFAULT_UI_FONT)
            .size(28)
            .style(|_| iced::widget::text::Style {
                color: Some(TEXT_PRIMARY)
            }),
        label_text("ポートフォリオ向け業務データダッシュボード"),
        glass_panel(controls),
        tabs,
        content,
        label_text(status),
    ]
    .spacing(12)
    .padding(20);

    container(shell)
        .style(space_background)
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
        styled_text_input("顧客フィルタ", &app.customer_query)
            .on_input(Message::CustomerQueryChanged)
            .padding(8)
    ]
    .spacing(8);
    for customer in customers {
        let label = format!(
            "{} | {} | {} | {} | {}",
            customer.customer_id,
            customer.full_name,
            status_label(customer.status.as_deref()),
            tier_label(customer.tier.as_deref()),
            country_label(customer.country.as_deref())
        );
        list = list.push(
            list_button(text(label.clone()).font(font_for_content(&label)))
                .width(Length::Fill)
                .on_press(Message::SelectCustomer(customer.customer_id.clone())),
        );
    }

    let detail = customer_detail_panel(app);
    row![
        glass_panel(scrollable(list)).width(Length::FillPortion(2)),
        glass_panel(detail).width(Length::FillPortion(3)),
    ]
    .spacing(12)
    .into()
}

fn customer_detail_panel(app: &App) -> Element<'_, Message> {
    let Some(detail) = &app.customer_detail else {
        return container(label_text("顧客を選択してください。"))
            .width(Length::Fill)
            .into();
    };

    let mut purchases = column![section_title("購入履歴")].spacing(4);
    if app.customer_purchases.is_empty() {
        purchases = purchases.push(body_text("- 購入履歴なし"));
    } else {
        for row in app.customer_purchases.iter().take(12) {
            purchases = purchases.push(body_text(format!(
                "{} | {} | {} ({}) x{} | 単価={} | 明細金額={} | {}",
                row.ordered_at,
                row.order_id,
                item_name_label(&row.item_name),
                row.item_id,
                row.quantity,
                money_opt(row.unit_price),
                money_opt(row.line_amount),
                order_status_label(Some(row.order_status.as_str()))
            )));
        }
    }

    let mut next_buy = column![section_title("次回購入候補")].spacing(4);
    if app.customer_next_buy.is_empty() {
        next_buy = next_buy.push(body_text("- 候補なし"));
    } else {
        for row in app.customer_next_buy.iter().take(10) {
            next_buy = next_buy.push(body_text(format!(
                "#{} {} ({}) score={:.3} as_of={}",
                row.rank,
                item_name_label(&row.item_name),
                row.item_id,
                row.score,
                row.as_of
            )));
        }
    }

    scrollable(
        column![
            section_title(format!("{} / {}", detail.customer_id, detail.full_name)).font(font_for_content(
                &format!("{} / {}", detail.customer_id, detail.full_name),
            )),
            body_text(format!(
                "ステータス={} | 会員ランク={} | 国籍={}",
                status_label(detail.status.as_deref()),
                tier_label(detail.tier.as_deref()),
                country_label(detail.country.as_deref())
            )),
            body_text(format!(
                "言語={} | 連絡先={} | マーケティング許可={}",
                language_label(detail.preferred_language.as_deref()),
                detail.email.as_deref().unwrap_or("-"),
                yes_no(detail.marketing_opt_in)
            )),
            body_text(format!(
                "累計売上={} | 注文回数={} | 最終購入={}",
                money_opt(detail.total_spend),
                detail.order_count.unwrap_or_default(),
                detail.last_purchase_date.as_deref().unwrap_or("-")
            )),
            row![
                body_text("地域=").font(DEFAULT_UI_FONT),
                text(detail.region.as_deref().unwrap_or("-").to_string())
                    .font(font_for_content(detail.region.as_deref().unwrap_or("-"))),
                body_text(" / ").font(DEFAULT_UI_FONT),
                text(detail.city.as_deref().unwrap_or("-").to_string())
                    .font(font_for_content(detail.city.as_deref().unwrap_or("-"))),
                body_text(" | 電話=").font(DEFAULT_UI_FONT),
                body_text(detail.phone.as_deref().unwrap_or("-").to_string()).font(DEFAULT_UI_FONT),
            ]
            .spacing(4),
            body_text(format!("備考={}", detail.notes.as_deref().unwrap_or("-"))),
            purchases,
            next_buy,
        ]
        .spacing(8),
    )
    .into()
}

fn inventory_view(app: &App) -> Element<'_, Message> {
    let list_header = row![
        styled_text_input("品目検索", &app.item_query)
            .on_input(Message::ItemQueryChanged)
            .padding(8)
            .width(Length::Fill),
        action_button("検索").on_press(Message::RefreshItems),
    ]
    .spacing(8);

    let mut list = column![list_header].spacing(8);
    for item in &app.items {
        list = list.push(
            list_button(
                text(format!(
                    "{} | {} | カテゴリ={} | ステータス={} | 在庫数={} | 発注残={} | 引当数={} | 更新={}",
                    item.item_id,
                    item_name_label(&item.item_name),
                    category_label(Some(item.category.as_str())),
                    active_label(item.is_active),
                    item.on_hand.unwrap_or_default(),
                    item.on_order.unwrap_or_default(),
                    item.reserved_qty.unwrap_or_default(),
                    item.updated_at.as_deref().unwrap_or("-")
                ))
                .font(DEFAULT_UI_FONT),
            )
            .width(Length::Fill)
            .on_press(Message::SelectItem(item.item_id.clone())),
        );
    }

    let detail = item_detail_panel(app);
    row![
        glass_panel(scrollable(list)).width(Length::FillPortion(2)),
        glass_panel(detail).width(Length::FillPortion(3)),
    ]
    .spacing(12)
    .into()
}

fn item_detail_panel(app: &App) -> Element<'_, Message> {
    let Some(detail) = &app.item_detail else {
        return container(label_text("品目を選択してください。"))
            .width(Length::Fill)
            .into();
    };
    let inventory = app.item_inventory.as_ref();
    let risk = app.item_risk.as_ref();

    container(
        column![
            section_title(format!("{} / {}", detail.item_id, item_name_label(&detail.item_name))),
            body_text(format!(
                "カテゴリ={} | ステータス={} | 単位={}",
                category_label(Some(detail.category.as_str())),
                active_label(detail.is_active),
                detail.uom.as_deref().unwrap_or("-")
            )),
            body_text(format!(
                "リードタイム={}日 | 最小発注数={} | ロットサイズ={}",
                detail.lead_time_days,
                detail.moq.unwrap_or_default(),
                detail.lot_size.unwrap_or_default()
            )),
            body_text(format!(
                "在庫数={} | 発注残={} | 引当数={} | 更新={} | 在庫品目={}",
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
            body_text(format!(
                "最新リスク={} | 推奨補充={} | 想定欠品={} | 平均在庫日数={}",
                risk_label(risk.and_then(|row| row.risk_level.as_deref())),
                risk.and_then(|row| row.recommended_reorder_qty)
                    .unwrap_or_default(),
                risk.and_then(|row| row.expected_stockout_qty)
                    .unwrap_or_default(),
                risk.and_then(|row| row.expected_days_on_hand)
                    .map(|value| format!("{value:.1}"))
                    .unwrap_or_else(|| "-".to_string())
            )),
            if let Some(risk) = risk {
                body_text(format!(
                    "参照実行={} / シナリオ={} ({}) / 実行依頼日時={}",
                    risk.run_id, risk.scenario_name, risk.scenario_id, risk.requested_at
                ))
            } else {
                body_text("シミュレーション結果はまだありません。")
            },
        ]
        .spacing(8),
    )
    .into()
}

fn simulations_view(app: &App) -> Element<'_, Message> {
    let controls = row![
        action_button("ベースライン実行").on_press(Message::RunSimulation),
        action_button("一覧更新").on_press(Message::RefreshSimulations),
    ]
    .spacing(8);

    let mut list = column![controls].spacing(8);
    for simulation in &app.simulations {
        list = list.push(
            list_button(
                text(format!(
                    "{} | {} | {} | ステータス={} | 実行依頼日時={} | 完了日時={} | レポートスキーマ={} | レポートURI={}",
                    simulation.run_id,
                    scenario_name_label(&simulation.scenario_name),
                    simulation.scenario_id,
                    simulation_status_label(Some(simulation.status.as_str())),
                    simulation.requested_at,
                    simulation.completed_at.as_deref().unwrap_or("-"),
                    simulation.report_schema_version.as_deref().unwrap_or("-"),
                    simulation.report_uri.as_deref().unwrap_or("-")
                ))
                .font(DEFAULT_UI_FONT),
            )
            .width(Length::Fill)
            .on_press(Message::SelectSimulation(simulation.run_id.clone())),
        );
    }

    let detail = simulation_detail_panel(app);
    row![
        glass_panel(scrollable(list)).width(Length::FillPortion(2)),
        glass_panel(detail).width(Length::FillPortion(3)),
    ]
    .spacing(12)
    .into()
}

fn simulation_detail_panel(app: &App) -> Element<'_, Message> {
    let Some(detail) = &app.simulation_detail else {
        return container(label_text("シミュレーションを選択してください。"))
            .width(Length::Fill)
            .into();
    };

    let report_summary = if let Some(report) = &app.simulation_report {
        let top_stockout = top_stockout_items(&report.inventory_series);
        column![
            body_text(format!(
                "スキーマ={} | 生成日時={}",
                report.schema_version.as_deref().unwrap_or("-"),
                report.generated_at
            )),
            body_text(format!(
                "資金期間={}..{} | 総入金={} | 総出金={} | 最終残高={}",
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
            body_text(format!(
                "シナリオ={} | 最小現金残高={} | 総欠品数={} | 欠品率={:.2}% | 平均在庫日数={:.2}",
                scenario_name_label(&report.scenario.name),
                with_commas(report.kpi.min_cash),
                report.kpi.total_stockout_qty,
                report.kpi.stockout_rate * 100.0,
                report.kpi.days_on_hand_avg
            )),
            body_text(format!(
                "資金データ点数={} | 在庫データ点数={}",
                report.cash_series.len(),
                report.inventory_series.len()
            )),
            body_text(format!(
                "在庫期間={}..{} | 総需要={} | 総販売数={}",
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
            section_title("主要アラート"),
            alerts_view(&report.alerts),
            section_title("欠品上位品目"),
            top_stockout,
        ]
        .spacing(6)
    } else {
        column![body_text("レポート JSON はまだありません。")].spacing(6)
    };

    scrollable(
        column![
            section_title(format!(
                "実行ID={} / {}",
                detail.run_id,
                scenario_name_label(&detail.scenario_name)
            )),
            body_text(format!(
                "ステータス={} | 実行依頼日時={} | レポート利用可={}",
                simulation_status_label(Some(detail.status.as_str())),
                detail.requested_at,
                yes_no(Some(detail.report_available))
            )),
            body_text(format!(
                "開始日時={} | 完了日時={}",
                detail.started_at.as_deref().unwrap_or("-"),
                detail.completed_at.as_deref().unwrap_or("-")
            )),
            body_text(format!(
                "スキーマ={} | レポートURI={} | シナリオID={}",
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
        return col.push(body_text("- アラートなし"));
    }
    for alert in alerts.iter().take(8) {
        col = col.push(body_text(format!(
            "{} | {} | {} | {} | {}",
            alert.date,
            severity_label(Some(alert.severity.as_str())),
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
        return col.push(body_text("- データなし"));
    }
    for (item_id, (stockout, on_hand, on_order)) in ranked.into_iter().take(8) {
        col = col.push(body_text(format!(
            "{} | 欠品数={} | 在庫数合計={} | 発注残合計={}",
            item_id, stockout, on_hand, on_order
        )));
    }
    col
}

fn tab_button<'a>(label: &'a str, tab: Tab, active: Tab) -> Element<'a, Message> {
    tab_styled_button(text(label.to_string()).font(DEFAULT_UI_FONT), tab == active)
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
        Some(true) => "はい",
        Some(false) => "いいえ",
        None => "-",
    }
}

fn country_label(value: Option<&str>) -> String {
    match value.unwrap_or("-") {
        "Japan" => "日本".to_string(),
        "China" => "中国".to_string(),
        "India" => "インド".to_string(),
        "United States" => "アメリカ合衆国".to_string(),
        "-" | "" => "-".to_string(),
        other => other.to_string(),
    }
}

fn language_label(value: Option<&str>) -> String {
    match value.unwrap_or("-") {
        "ja" | "ja-JP" => "日本語".to_string(),
        "en" => "英語".to_string(),
        "en-US" => "英語（米国）".to_string(),
        "zh" => "中国語".to_string(),
        "zh-CN" => "中国語（簡体字）".to_string(),
        "hi" => "ヒンディー語".to_string(),
        "fr-FR" => "フランス語".to_string(),
        "ar" | "ar-SA" => "アラビア語".to_string(),
        "-" | "" => "-".to_string(),
        other => other.to_string(),
    }
}

fn status_label(value: Option<&str>) -> String {
    match value.unwrap_or("-") {
        "active" => "アクティブ".to_string(),
        "inactive" => "非アクティブ".to_string(),
        "pending" => "保留中".to_string(),
        "banned" => "利用停止".to_string(),
        "-" | "" => "-".to_string(),
        other => other.to_string(),
    }
}

fn tier_label(value: Option<&str>) -> String {
    match value.unwrap_or("-") {
        "bronze" => "ブロンズ".to_string(),
        "silver" => "シルバー".to_string(),
        "gold" => "ゴールド".to_string(),
        "platinum" => "プラチナ".to_string(),
        "-" | "" => "-".to_string(),
        other => other.to_string(),
    }
}

fn category_label(value: Option<&str>) -> String {
    match value.unwrap_or("-") {
        "apparel" => "アパレル".to_string(),
        "beauty" => "ビューティー".to_string(),
        "electronics" => "エレクトロニクス".to_string(),
        "food" => "食品".to_string(),
        "home" => "ホーム".to_string(),
        "office" => "オフィス".to_string(),
        "outdoor" => "アウトドア".to_string(),
        "pet" => "ペット".to_string(),
        "sports" => "スポーツ".to_string(),
        "test" => "テスト".to_string(),
        "wellness" => "ウェルネス".to_string(),
        "-" | "" => "-".to_string(),
        other => other.to_string(),
    }
}

fn item_name_label(value: &str) -> String {
    let Some((category, number)) = value.split_once(" product ") else {
        return value.to_string();
    };

    let category = category_label(Some(category));
    if category == "-" {
        value.to_string()
    } else {
        format!("{category}商品 {number}")
    }
}

fn active_label(value: bool) -> &'static str {
    if value { "アクティブ" } else { "非アクティブ" }
}

fn order_status_label(value: Option<&str>) -> String {
    match value.unwrap_or("-") {
        "pending" => "保留中".to_string(),
        "paid" => "支払済み".to_string(),
        "shipped" => "発送済み".to_string(),
        "completed" => "完了".to_string(),
        "cancelled" | "canceled" => "キャンセル".to_string(),
        "returned" => "返品".to_string(),
        "-" | "" => "-".to_string(),
        other => other.to_string(),
    }
}

fn simulation_status_label(value: Option<&str>) -> String {
    match value.unwrap_or("-") {
        "running" => "実行中".to_string(),
        "succeeded" => "成功".to_string(),
        "failed" => "失敗".to_string(),
        "queued" => "待機中".to_string(),
        "cancelled" | "canceled" => "キャンセル".to_string(),
        "-" | "" => "-".to_string(),
        other => other.to_string(),
    }
}

fn scenario_name_label(value: &str) -> String {
    match value {
        "Integration Test Baseline" => "統合テスト ベースライン".to_string(),
        "Baseline Docker" => "Docker ベースライン".to_string(),
        "Baseline Local" => "ローカル ベースライン".to_string(),
        "full dataset verification run" => "全件データ検証実行".to_string(),
        other if other.contains("Baseline") => other.replace("Baseline", "ベースライン"),
        other => other.to_string(),
    }
}

fn risk_label(value: Option<&str>) -> String {
    match value.unwrap_or("-") {
        "low" => "低リスク".to_string(),
        "medium" => "中リスク".to_string(),
        "high" => "高リスク".to_string(),
        "critical" => "重大リスク".to_string(),
        "-" | "" => "未計算".to_string(),
        other => other.to_string(),
    }
}

fn severity_label(value: Option<&str>) -> String {
    match value.unwrap_or("-") {
        "info" => "情報".to_string(),
        "warning" | "warn" => "警告".to_string(),
        "error" => "エラー".to_string(),
        "critical" => "重大".to_string(),
        "-" | "" => "-".to_string(),
        other => other.to_string(),
    }
}

fn space_background(_theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        text_color: Some(TEXT_PRIMARY),
        background: Some(Background::Gradient(
            iced::gradient::Linear::new(Degrees(135.0))
                .add_stop(0.0, Color::from_rgb(0.015, 0.018, 0.050))
                .add_stop(0.34, Color::from_rgb(0.065, 0.045, 0.190))
                .add_stop(0.68, Color::from_rgb(0.025, 0.100, 0.170))
                .add_stop(1.0, Color::from_rgb(0.008, 0.010, 0.026))
                .into(),
        )),
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

fn panel_style(_theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        text_color: Some(TEXT_PRIMARY),
        background: Some(Background::Color(PANEL_BG)),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: BORDER_SUBTLE,
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
            offset: Vector::new(0.0, 12.0),
            blur_radius: 28.0,
        },
    }
}

fn glass_panel<'a>(
    content: impl Into<Element<'a, Message>>,
) -> iced::widget::Container<'a, Message> {
    container(content).padding(12).style(panel_style)
}

fn input_style(_theme: &Theme, status: iced::widget::text_input::Status) -> iced::widget::text_input::Style {
    let border_color = match status {
        iced::widget::text_input::Status::Focused => ACCENT_BRIGHT,
        iced::widget::text_input::Status::Hovered => Color::from_rgba(0.70, 0.76, 1.0, 0.48),
        _ => BORDER_SUBTLE,
    };

    iced::widget::text_input::Style {
        background: Background::Color(Color::from_rgba(0.035, 0.040, 0.095, 0.92)),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: border_color,
        },
        icon: TEXT_MUTED,
        placeholder: TEXT_MUTED,
        value: TEXT_PRIMARY,
        selection: ACCENT,
    }
}

fn styled_text_input<'a>(
    placeholder: &'a str,
    value: &'a str,
) -> iced::widget::TextInput<'a, Message> {
    text_input(placeholder, value).style(input_style)
}

fn button_style(_theme: &Theme, status: iced::widget::button::Status) -> iced::widget::button::Style {
    let (background, border, text_color) = match status {
        iced::widget::button::Status::Hovered => (
            Color::from_rgba(0.43, 0.30, 0.95, 0.96),
            ACCENT_BRIGHT,
            Color::WHITE,
        ),
        iced::widget::button::Status::Pressed => (
            Color::from_rgba(0.24, 0.18, 0.70, 0.98),
            ACCENT,
            Color::WHITE,
        ),
        iced::widget::button::Status::Disabled => (
            Color::from_rgba(0.10, 0.11, 0.18, 0.65),
            BORDER_SUBTLE,
            TEXT_MUTED,
        ),
        iced::widget::button::Status::Active => (
            Color::from_rgba(0.20, 0.16, 0.48, 0.95),
            Color::from_rgba(0.78, 0.68, 1.0, 0.36),
            TEXT_PRIMARY,
        ),
    };

    iced::widget::button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border {
            radius: 7.0.into(),
            width: 1.0,
            color: border,
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.28),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
    }
}

fn list_button_style(_theme: &Theme, status: iced::widget::button::Status) -> iced::widget::button::Style {
    let background = match status {
        iced::widget::button::Status::Hovered => Color::from_rgba(0.20, 0.24, 0.46, 0.95),
        iced::widget::button::Status::Pressed => Color::from_rgba(0.16, 0.18, 0.36, 0.98),
        _ => Color::from_rgba(0.10, 0.13, 0.26, 0.82),
    };

    iced::widget::button::Style {
        background: Some(Background::Color(background)),
        text_color: TEXT_PRIMARY,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: BORDER_SUBTLE,
        },
        shadow: Shadow::default(),
    }
}

fn tab_button_style(active: bool) -> impl Fn(&Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_theme, status| {
        let background = if active {
            match status {
                iced::widget::button::Status::Hovered => Color::from_rgba(0.30, 0.42, 0.95, 0.96),
                _ => Color::from_rgba(0.44, 0.24, 0.95, 0.95),
            }
        } else {
            match status {
                iced::widget::button::Status::Hovered => Color::from_rgba(0.16, 0.20, 0.38, 0.94),
                _ => Color::from_rgba(0.07, 0.09, 0.18, 0.78),
            }
        };

        iced::widget::button::Style {
            background: Some(Background::Color(background)),
            text_color: TEXT_PRIMARY,
            border: Border {
                radius: 7.0.into(),
                width: 1.0,
                color: if active { ACCENT_BRIGHT } else { BORDER_SUBTLE },
            },
            shadow: Shadow::default(),
        }
    }
}

fn action_button<'a>(label: &'a str) -> iced::widget::Button<'a, Message> {
    button(text(label).font(DEFAULT_UI_FONT))
        .padding([8, 12])
        .style(button_style)
}

fn list_button<'a>(content: impl Into<Element<'a, Message>>) -> iced::widget::Button<'a, Message> {
    button(content).padding([9, 10]).style(list_button_style)
}

fn tab_styled_button<'a>(
    content: impl Into<Element<'a, Message>>,
    active: bool,
) -> iced::widget::Button<'a, Message> {
    button(content).padding([9, 14]).style(tab_button_style(active))
}

fn label_text<'a>(content: impl Into<String>) -> iced::widget::Text<'a> {
    text(content.into())
        .font(DEFAULT_UI_FONT)
        .size(15)
        .style(|_| iced::widget::text::Style {
            color: Some(TEXT_MUTED),
        })
}

fn body_text<'a>(content: impl Into<String>) -> iced::widget::Text<'a> {
    text(content.into())
        .font(DEFAULT_UI_FONT)
        .size(15)
        .style(|_| iced::widget::text::Style {
            color: Some(TEXT_PRIMARY),
        })
}

fn section_title<'a>(content: impl Into<String>) -> iced::widget::Text<'a> {
    text(content.into())
        .font(DEFAULT_UI_FONT)
        .size(17)
        .style(|_| iced::widget::text::Style {
            color: Some(Color::from_rgb(0.82, 0.88, 1.0)),
        })
}
