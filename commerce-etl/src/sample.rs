use crate::config::SampleConfig;
use crate::csv_input::{InventoryRow, ItemRow, OrderItemRow, OrderRow};
use chrono::{Duration, NaiveDate};
use csv::{ReaderBuilder, Trim, Writer};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::Path;

const CATEGORY_NAMES: [&str; 10] = [
    "apparel",
    "beauty",
    "electronics",
    "food",
    "home",
    "office",
    "outdoor",
    "pet",
    "sports",
    "wellness",
];

const ITEM_POPULARITY_WEIGHTS: [u32; 10] = [25, 18, 14, 11, 9, 7, 6, 4, 3, 3];

#[derive(Debug, Clone, Deserialize)]
struct CustomerSeedRow {
    #[serde(rename = "CustomerID")]
    customer_id: String,
    country: Option<String>,
    status: Option<String>,
    tier: Option<String>,
    preferred_language: Option<String>,
}

#[derive(Debug, Clone)]
struct CustomerProfile {
    row: CustomerSeedRow,
    propensity: i32,
    primary_category: usize,
    secondary_category: usize,
}

#[derive(Debug, Clone)]
struct ItemCatalogEntry {
    row: ItemRow,
    category_index: usize,
    base_price: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct SampleSummary {
    pub customers: usize,
    pub items: usize,
    pub orders: usize,
    pub order_items: usize,
    pub repeat_customers: usize,
}

#[derive(Debug, Serialize)]
struct SampleMetadata {
    customers: usize,
    items: usize,
    orders: usize,
    order_items: usize,
    repeat_customers: usize,
    seed: u64,
    generated_files: BTreeMap<&'static str, String>,
}

pub fn generate_sample_files(
    config: &SampleConfig,
) -> Result<SampleSummary, Box<dyn std::error::Error>> {
    let customers = load_customer_seed(&config.customers_csv, config.customer_count)?;
    if customers.len() != config.customer_count {
        return Err(format!(
            "顧客件数が不足しています。期待値={}, 実際={}",
            config.customer_count,
            customers.len()
        )
        .into());
    }

    fs::create_dir_all(&config.output_dir)?;
    let mut rng = DeterministicRng::new(config.seed);
    let items = build_items(config.item_count, &mut rng);
    let order_counts = assign_order_counts(&customers, config.order_count);
    let repeat_customers = order_counts.iter().filter(|count| **count >= 2).count();
    let (orders, order_items) = build_orders(&customers, &items, &order_counts, &mut rng)?;
    let inventory = build_inventory(&items, &order_items);

    let items_path = config.output_dir.join("items.csv");
    let orders_path = config.output_dir.join("orders.csv");
    let order_items_path = config.output_dir.join("order_items.csv");
    let inventory_path = config.output_dir.join("inventory.csv");
    let metadata_path = config.output_dir.join("sample_metadata.json");

    write_csv(&items_path, items.iter().map(|entry| &entry.row))?;
    write_csv(&orders_path, orders.iter())?;
    write_csv(&order_items_path, order_items.iter())?;
    write_csv(&inventory_path, inventory.iter())?;

    let metadata = SampleMetadata {
        customers: customers.len(),
        items: items.len(),
        orders: orders.len(),
        order_items: order_items.len(),
        repeat_customers,
        seed: config.seed,
        generated_files: BTreeMap::from([
            ("items", items_path.display().to_string()),
            ("orders", orders_path.display().to_string()),
            ("order_items", order_items_path.display().to_string()),
            ("inventory", inventory_path.display().to_string()),
        ]),
    };
    fs::write(metadata_path, serde_json::to_string_pretty(&metadata)?)?;

    Ok(SampleSummary {
        customers: customers.len(),
        items: items.len(),
        orders: orders.len(),
        order_items: order_items.len(),
        repeat_customers,
    })
}

fn load_customer_seed(
    path: &Path,
    customer_count: usize,
) -> Result<Vec<CustomerProfile>, Box<dyn std::error::Error>> {
    let mut reader = ReaderBuilder::new().trim(Trim::All).from_path(path)?;
    let mut profiles = Vec::with_capacity(customer_count);
    for row in reader.deserialize::<CustomerSeedRow>().take(customer_count) {
        let row = row?;
        let country_code = row.country.as_deref().unwrap_or_default();
        let status = row.status.as_deref().unwrap_or_default();
        let tier = row.tier.as_deref().unwrap_or_default();
        let language = row.preferred_language.as_deref().unwrap_or_default();
        let seed = stable_hash(&format!(
            "{}|{}|{}|{}|{}",
            row.customer_id, country_code, status, tier, language
        ));
        let primary_category = (seed as usize) % CATEGORY_NAMES.len();
        let secondary_category =
            ((seed >> 8) as usize + primary_category + 3) % CATEGORY_NAMES.len();
        let propensity = status_weight(status)
            + tier_weight(tier)
            + country_weight(country_code)
            + (((seed >> 16) & 0x7) as i32);
        profiles.push(CustomerProfile {
            row,
            propensity,
            primary_category,
            secondary_category,
        });
    }
    profiles.sort_by(|a, b| {
        b.propensity
            .cmp(&a.propensity)
            .then(a.row.customer_id.cmp(&b.row.customer_id))
    });
    Ok(profiles)
}

fn build_items(item_count: usize, rng: &mut DeterministicRng) -> Vec<ItemCatalogEntry> {
    let count = item_count.max(1);
    let mut items = Vec::with_capacity(count);
    for idx in 0..count {
        let category_index = idx % CATEGORY_NAMES.len();
        let category = CATEGORY_NAMES[category_index].to_string();
        let item_number = idx + 1;
        let lead_time_days = 2 + (category_index as i32 % 5) + (rng.next_u32() % 4) as i32;
        let base_price =
            category_base_price(category_index) + f64::from((rng.next_u32() % 250) as u16);
        items.push(ItemCatalogEntry {
            row: ItemRow {
                item_id: format!("ITEM{:04}", item_number),
                item_name: format!("{} product {:03}", category, item_number),
                category,
                uom: Some("ea".to_string()),
                is_active: Some(true),
                lead_time_days: Some(lead_time_days),
                moq: Some(5 + (category_index as i32 % 4) * 5),
                lot_size: Some(10 + (category_index as i32 % 5) * 5),
            },
            category_index,
            base_price,
        });
    }
    items
}

fn assign_order_counts(customers: &[CustomerProfile], target_orders: usize) -> Vec<usize> {
    let customer_count = customers.len();
    let mut counts = vec![1_usize; customer_count];
    let mut remaining = target_orders.saturating_sub(customer_count);
    let max_orders = (0..customer_count)
        .map(|idx| {
            if idx < customer_count / 5 {
                8
            } else if idx < customer_count / 2 {
                5
            } else if idx < customer_count * 4 / 5 {
                3
            } else {
                2
            }
        })
        .collect::<Vec<_>>();

    while remaining > 0 {
        let mut progressed = false;
        for idx in 0..customer_count {
            if remaining == 0 {
                break;
            }
            if counts[idx] < max_orders[idx] {
                counts[idx] += 1;
                remaining -= 1;
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }
    counts
}

fn build_orders(
    customers: &[CustomerProfile],
    items: &[ItemCatalogEntry],
    order_counts: &[usize],
    rng: &mut DeterministicRng,
) -> Result<(Vec<OrderRow>, Vec<OrderItemRow>), Box<dyn std::error::Error>> {
    let mut orders = Vec::new();
    let mut order_items = Vec::new();
    let start_date =
        NaiveDate::from_ymd_opt(2024, 1, 1).ok_or_else(|| "開始日を生成できません".to_string())?;
    let catalog_by_category = build_catalog_index(items);

    let mut order_sequence = 1_u64;
    for (customer_idx, customer) in customers.iter().enumerate() {
        let count = order_counts[customer_idx];
        let spacing = (720_i64 / (count as i64 + 1)).max(7);
        let base_offset = (stable_hash(&customer.row.customer_id) % 60) as i64;
        let mut previous_category = customer.primary_category;

        for order_idx in 0..count {
            let date = start_date
                + Duration::days(
                    base_offset + spacing * order_idx as i64 + (rng.next_u32() % 5) as i64,
                );
            let ordered_at = format!(
                "{}T{:02}:{:02}:00+09:00",
                date.format("%Y-%m-%d"),
                9 + (rng.next_u32() % 9),
                rng.next_u32() % 60
            );

            let line_count = sample_line_count(rng);
            let order_id = format!("ORD{:08}", order_sequence);
            order_sequence += 1;

            let mut chosen_items = BTreeSet::new();
            let mut total_amount = 0.0;
            let mut first_category = None;
            for line_no in 1..=line_count {
                let category = choose_category(customer, previous_category, rng);
                let entry = choose_item(items, &catalog_by_category, category, rng);
                first_category.get_or_insert(category);

                if !chosen_items.insert(entry.row.item_id.clone()) {
                    continue;
                }
                let quantity = sample_quantity(rng);
                let unit_price = adjust_price(entry.base_price, customer, rng);
                let line_amount = unit_price * quantity as f64;
                total_amount += line_amount;

                order_items.push(OrderItemRow {
                    order_id: order_id.clone(),
                    line_no: line_no as i32,
                    item_id: entry.row.item_id.clone(),
                    quantity,
                    unit_price: Some(round2(unit_price)),
                    line_amount: Some(round2(line_amount)),
                });
            }

            previous_category = first_category.unwrap_or(previous_category);
            orders.push(OrderRow {
                order_id,
                customer_id: customer.row.customer_id.clone(),
                ordered_at,
                status: order_status(order_idx, count).to_string(),
                currency: Some("JPY".to_string()),
                total_amount: Some(round2(total_amount)),
            });
        }
    }

    Ok((orders, order_items))
}

fn build_inventory(items: &[ItemCatalogEntry], order_items: &[OrderItemRow]) -> Vec<InventoryRow> {
    let mut totals = HashMap::<String, i32>::new();
    for row in order_items {
        *totals.entry(row.item_id.clone()).or_insert(0) += row.quantity;
    }

    items
        .iter()
        .map(|item| {
            let total_qty = totals.get(&item.row.item_id).copied().unwrap_or(0).max(1);
            let monthly_avg = (total_qty as f64 / 24.0).ceil() as i32;
            let on_hand = (monthly_avg * 2).max(20);
            let on_order = (monthly_avg / 2).max(5);
            let reserved_qty = (monthly_avg / 5).max(0);
            InventoryRow {
                item_id: item.row.item_id.clone(),
                on_hand,
                on_order: Some(on_order),
                reserved_qty: Some(reserved_qty),
            }
        })
        .collect()
}

fn build_catalog_index(items: &[ItemCatalogEntry]) -> Vec<Vec<usize>> {
    let mut by_category = vec![Vec::new(); CATEGORY_NAMES.len()];
    for (idx, item) in items.iter().enumerate() {
        by_category[item.category_index].push(idx);
    }
    by_category
}

fn choose_category(
    customer: &CustomerProfile,
    previous_category: usize,
    rng: &mut DeterministicRng,
) -> usize {
    let roll = rng.next_u32() % 100;
    if roll < 45 {
        previous_category
    } else if roll < 80 {
        customer.primary_category
    } else if roll < 95 {
        customer.secondary_category
    } else {
        (rng.next_u32() as usize) % CATEGORY_NAMES.len()
    }
}

fn choose_item<'a>(
    items: &'a [ItemCatalogEntry],
    by_category: &[Vec<usize>],
    category: usize,
    rng: &mut DeterministicRng,
) -> &'a ItemCatalogEntry {
    let item_indexes = &by_category[category];
    let weight_total = ITEM_POPULARITY_WEIGHTS
        .iter()
        .take(item_indexes.len())
        .sum::<u32>();
    let mut target = rng.next_u32() % weight_total.max(1);
    for (position, item_idx) in item_indexes.iter().enumerate() {
        let weight = ITEM_POPULARITY_WEIGHTS[position];
        if target < weight {
            return &items[*item_idx];
        }
        target -= weight;
    }
    &items[*item_indexes.last().unwrap_or(&0)]
}

fn sample_line_count(rng: &mut DeterministicRng) -> usize {
    let roll = rng.next_u32() % 100;
    match roll {
        0..=14 => 1,
        15..=39 => 2,
        40..=69 => 3,
        70..=89 => 4,
        _ => 5,
    }
}

fn sample_quantity(rng: &mut DeterministicRng) -> i32 {
    let roll = rng.next_u32() % 100;
    match roll {
        0..=69 => 1,
        70..=89 => 2,
        90..=97 => 3,
        _ => 4,
    }
}

fn adjust_price(base_price: f64, customer: &CustomerProfile, rng: &mut DeterministicRng) -> f64 {
    let tier_factor = match customer.row.tier.as_deref().unwrap_or_default() {
        "platinum" => 1.15,
        "gold" => 1.08,
        "silver" => 1.0,
        "bronze" => 0.93,
        _ => 1.0,
    };
    let random_factor = 0.92 + (rng.next_u32() % 17) as f64 / 100.0;
    base_price * tier_factor * random_factor
}

fn order_status(order_idx: usize, total_orders: usize) -> &'static str {
    if order_idx + 1 == total_orders {
        "completed"
    } else {
        "shipped"
    }
}

fn category_base_price(category_index: usize) -> f64 {
    match category_index {
        0 => 4200.0,
        1 => 3200.0,
        2 => 12800.0,
        3 => 1800.0,
        4 => 5400.0,
        5 => 3800.0,
        6 => 7600.0,
        7 => 2600.0,
        8 => 6900.0,
        _ => 4100.0,
    }
}

fn tier_weight(tier: &str) -> i32 {
    match tier {
        "platinum" => 8,
        "gold" => 6,
        "silver" => 4,
        "bronze" => 2,
        _ => 1,
    }
}

fn status_weight(status: &str) -> i32 {
    match status {
        "active" => 8,
        "pending" => 4,
        "inactive" => 2,
        "banned" => 0,
        _ => 1,
    }
}

fn country_weight(country: &str) -> i32 {
    match country {
        "Japan" => 5,
        "United States" => 4,
        _ => 2,
    }
}

fn stable_hash(value: &str) -> u64 {
    let mut hash = 1469598103934665603_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn write_csv<'a, T>(
    path: &Path,
    rows: impl IntoIterator<Item = &'a T>,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Serialize + 'a,
{
    let mut writer = Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

#[derive(Debug, Clone)]
struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        let state = if seed == 0 { 0x9E3779B97F4A7C15 } else { seed };
        Self { state }
    }

    fn next_u32(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        (self.state >> 16) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::{SampleConfig, generate_sample_files};
    use std::fs;

    #[test]
    fn generates_expected_counts_for_small_fixture() {
        let temp = std::env::temp_dir().join("decision-pack-commerce-etl-sample-test");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let customers_csv = temp.join("customers.csv");
        fs::write(
            &customers_csv,
            "CustomerID,country,status,tier,preferred_language\nC1,Japan,active,gold,ja\nC2,United States,active,silver,en\nC3,Japan,inactive,bronze,ja\nC4,Japan,active,platinum,ja\n",
        )
        .unwrap();

        let output = temp.join("out");
        let summary = generate_sample_files(&SampleConfig {
            customers_csv,
            output_dir: output.clone(),
            customer_count: 4,
            item_count: 10,
            order_count: 12,
            seed: 42,
        })
        .unwrap();

        assert_eq!(summary.customers, 4);
        assert_eq!(summary.items, 10);
        assert_eq!(summary.orders, 12);
        assert!(summary.order_items >= 12);
        assert!(output.join("items.csv").exists());
        assert!(output.join("sample_metadata.json").exists());
        let _ = fs::remove_dir_all(&temp);
    }
}
