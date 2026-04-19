//! 未整形の顧客 raw サンプルを生成します。
//!
//! 推薦評価と ETL 耐久確認の両方に使えるよう、英語・日本語・ヒンディー語・中国語の
//! 氏名と、各国に対応した住所・電話番号・郵便番号を含む顧客 CSV を出力します。

use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use chrono::{Duration, NaiveDate};
use csv::WriterBuilder;
use serde::Serialize;

use crate::config::GenerateRawSampleConfig;

const DIRTY_HEADER_ROW: [&str; 19] = [
    " CustomerID ",
    "full_name",
    "email",
    "phone",
    "address_line",
    "city",
    "region",
    "postal_code",
    "country",
    "birth_date",
    "signup_date",
    "last_purchase_date",
    "status",
    "tier",
    "preferred_language",
    "marketing_opt_in",
    "total_spend",
    "order_count",
    "notes",
];

const US_FIRST_NAMES: &[&str] = &[
    "Ava", "Liam", "Noah", "Emma", "Olivia", "Sophia", "Mia", "Ethan", "Lucas", "Harper", "Ella",
    "Grace", "Henry", "Jack", "Nora", "Chloe",
];
const US_LAST_NAMES: &[&str] = &[
    "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Taylor",
    "Wilson", "Moore", "Anderson", "Thomas", "Jackson", "White", "Harris",
];
const US_LOCATIONS: &[(&str, &str)] = &[
    ("Seattle", "WA"),
    ("Austin", "TX"),
    ("Boston", "MA"),
    ("Denver", "CO"),
    ("Miami", "FL"),
    ("Chicago", "IL"),
    ("Portland", "OR"),
    ("Newark", "NJ"),
];
const US_STREET_NAMES: &[&str] = &[
    "Oak", "Maple", "Pine", "Cedar", "Elm", "Lakeview", "Sunset", "Willow", "River", "Madison",
];

const JP_FAMILY_NAMES: &[&str] = &[
    "佐藤", "鈴木", "高橋", "田中", "伊藤", "渡辺", "山本", "中村", "小林", "加藤", "吉田", "山田",
];
const JP_GIVEN_NAMES: &[&str] = &[
    "太郎", "花子", "美咲", "大翔", "結衣", "健太", "彩乃", "蓮", "陽菜", "悠真", "琴音", "大輝",
];
const JP_LOCATIONS: &[(&str, &str, &str)] = &[
    ("新宿区", "東京都", "西新宿"),
    ("渋谷区", "東京都", "恵比寿"),
    ("大阪市北区", "大阪府", "梅田"),
    ("福岡市中央区", "福岡県", "天神"),
    ("札幌市中央区", "北海道", "大通西"),
    ("名古屋市中区", "愛知県", "栄"),
];

const IN_GIVEN_NAMES: &[&str] = &[
    "आरव",
    "विवान",
    "आदित्य",
    "सिया",
    "अनया",
    "काव्या",
    "मीरा",
    "आर्या",
    "इशान",
    "कबीर",
];
const IN_FAMILY_NAMES: &[&str] = &[
    "शर्मा",
    "वर्मा",
    "गुप्ता",
    "सिंह",
    "पटेल",
    "यादव",
    "मिश्रा",
    "कुमार",
    "दास",
    "जोशी",
];
const IN_LOCATIONS: &[(&str, &str, &str)] = &[
    ("दिल्ली", "दिल्ली", "एम.जी. रोड"),
    ("मुंबई", "महाराष्ट्र", "लिंक रोड"),
    ("जयपुर", "राजस्थान", "टोंक रोड"),
    ("लखनऊ", "उत्तर प्रदेश", "हजरतगंज"),
    ("भोपाल", "मध्य प्रदेश", "न्यू मार्केट"),
    ("पटना", "बिहार", "बोरिंग रोड"),
];

const CN_FAMILY_NAMES: &[&str] = &[
    "王", "李", "张", "刘", "陈", "杨", "黄", "赵", "周", "吴", "徐", "孙",
];
const CN_GIVEN_NAMES: &[&str] = &[
    "小明", "晓雪", "伟", "静", "宇航", "佳怡", "晨曦", "子涵", "建国", "思雨", "浩然", "欣怡",
];
const CN_LOCATIONS: &[(&str, &str, &str)] = &[
    ("北京市", "北京市", "朝阳区建国路"),
    ("上海市", "上海市", "浦东新区世纪大道"),
    ("广州市", "广东省", "天河路"),
    ("深圳市", "广东省", "南山区科技园"),
    ("杭州市", "浙江省", "西湖区文三路"),
    ("成都市", "四川省", "高新区天府大道"),
];

const STATUS_VALUES: &[&str] = &["ACTIVE", "inactive", "Pending", "banned"];
const TIER_VALUES: &[&str] = &["bronze", "silv", "Gold", "platinum"];
const MARKETING_VALUES: &[&str] = &["1", "0", "yes", "no", "TRUE", "false", "Y", "N"];
const NOTE_VALUES: &[&str] = &[
    "",
    "legacy import",
    "needs follow-up call",
    "prefers invoice by PDF",
    "VIP segment candidate",
    "coupon used in previous order",
    "会員ランク再確認",
    "需要予測の観察対象",
];

#[derive(Debug, Clone, Serialize)]
pub struct GeneratedRawSampleSummary {
    pub output_raw: String,
    pub output_metadata: String,
    pub target_formatted_count: usize,
    pub raw_rows_written: usize,
    pub embedded_headers_written: usize,
    pub invalid_rows_written: usize,
    pub country_counts: BTreeMap<String, usize>,
    pub language_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize)]
struct RawCustomerRow {
    #[serde(rename = " CustomerID ")]
    customer_id: String,
    full_name: String,
    email: String,
    phone: String,
    address_line: String,
    city: String,
    region: String,
    postal_code: String,
    country: String,
    birth_date: String,
    signup_date: String,
    last_purchase_date: String,
    status: String,
    tier: String,
    preferred_language: String,
    marketing_opt_in: String,
    total_spend: String,
    order_count: String,
    notes: String,
}

#[derive(Debug, Clone, Copy)]
enum LocaleProfile {
    UsEnglish,
    Japanese,
    Hindi,
    Chinese,
}

pub fn generate_raw_sample(
    config: &GenerateRawSampleConfig,
) -> Result<GeneratedRawSampleSummary, Box<dyn Error>> {
    ensure_parent_dir(&config.output_raw)?;

    let file = File::create(&config.output_raw)?;
    let mut writer = WriterBuilder::new().has_headers(false).from_writer(file);
    writer.write_record(DIRTY_HEADER_ROW)?;

    let mut rng = DeterministicRng::new(config.seed);
    let mut country_counts = BTreeMap::new();
    let mut language_counts = BTreeMap::new();
    let mut embedded_headers_written = 0usize;

    for index in 0..config.target_formatted_count {
        let locale = choose_locale(&mut rng);
        let row = build_valid_row(index, locale, &mut rng);
        *country_counts
            .entry(normalized_country(locale).to_string())
            .or_insert(0) += 1;
        *language_counts
            .entry(normalized_language(locale).to_string())
            .or_insert(0) += 1;
        writer.serialize(row)?;

        if (index + 1) % 10_000 == 0 && index + 1 != config.target_formatted_count {
            writer.write_record(DIRTY_HEADER_ROW)?;
            embedded_headers_written += 1;
        }
    }

    for invalid_index in 0..config.invalid_row_count {
        writer.serialize(build_invalid_row(invalid_index, &mut rng))?;
    }

    writer.flush()?;

    let summary = GeneratedRawSampleSummary {
        output_raw: config.output_raw.to_string_lossy().to_string(),
        output_metadata: metadata_path(&config.output_raw)
            .to_string_lossy()
            .to_string(),
        target_formatted_count: config.target_formatted_count,
        raw_rows_written: config.target_formatted_count
            + embedded_headers_written
            + config.invalid_row_count,
        embedded_headers_written,
        invalid_rows_written: config.invalid_row_count,
        country_counts,
        language_counts,
    };

    fs::write(
        metadata_path(&config.output_raw),
        serde_json::to_string_pretty(&summary)?,
    )?;

    Ok(summary)
}

fn ensure_parent_dir(path: &Path) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn metadata_path(output_raw: &Path) -> PathBuf {
    let stem = output_raw
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("raw_customers");
    output_raw.with_file_name(format!("{stem}_metadata.json"))
}

fn choose_locale(rng: &mut DeterministicRng) -> LocaleProfile {
    match rng.range_usize(100) {
        0..=27 => LocaleProfile::UsEnglish,
        28..=55 => LocaleProfile::Japanese,
        56..=77 => LocaleProfile::Hindi,
        _ => LocaleProfile::Chinese,
    }
}

fn normalized_country(locale: LocaleProfile) -> &'static str {
    match locale {
        LocaleProfile::UsEnglish => "United States",
        LocaleProfile::Japanese => "Japan",
        LocaleProfile::Hindi => "India",
        LocaleProfile::Chinese => "China",
    }
}

fn normalized_language(locale: LocaleProfile) -> &'static str {
    match locale {
        LocaleProfile::UsEnglish => "en-US",
        LocaleProfile::Japanese => "ja",
        LocaleProfile::Hindi => "hi",
        LocaleProfile::Chinese => "zh-CN",
    }
}

fn build_valid_row(
    index: usize,
    locale: LocaleProfile,
    rng: &mut DeterministicRng,
) -> RawCustomerRow {
    let customer_id = format!("CUST{:06}", index + 1);
    let signup_date = date_from_offset(
        NaiveDate::from_ymd_opt(2018, 1, 1).unwrap(),
        rng.range_i64(2_700),
    );
    let birth_date = date_from_offset(
        NaiveDate::from_ymd_opt(1955, 1, 1).unwrap(),
        rng.range_i64(18_000),
    );
    let status = STATUS_VALUES[rng.range_usize(STATUS_VALUES.len())];
    let tier = TIER_VALUES[rng.range_usize(TIER_VALUES.len())];
    let order_count = derive_order_count(status, rng);
    let total_spend = derive_total_spend(order_count, tier, rng);
    let last_purchase_date = derive_last_purchase_date(signup_date, order_count, rng);
    let marketing_opt_in = MARKETING_VALUES[rng.range_usize(MARKETING_VALUES.len())];
    let notes = NOTE_VALUES[rng.range_usize(NOTE_VALUES.len())].to_string();

    match locale {
        LocaleProfile::UsEnglish => build_us_row(
            customer_id,
            birth_date,
            signup_date,
            last_purchase_date,
            status,
            tier,
            marketing_opt_in,
            total_spend,
            order_count,
            notes,
            rng,
        ),
        LocaleProfile::Japanese => build_jp_row(
            customer_id,
            birth_date,
            signup_date,
            last_purchase_date,
            status,
            tier,
            marketing_opt_in,
            total_spend,
            order_count,
            notes,
            rng,
        ),
        LocaleProfile::Hindi => build_in_row(
            customer_id,
            birth_date,
            signup_date,
            last_purchase_date,
            status,
            tier,
            marketing_opt_in,
            total_spend,
            order_count,
            notes,
            rng,
        ),
        LocaleProfile::Chinese => build_cn_row(
            customer_id,
            birth_date,
            signup_date,
            last_purchase_date,
            status,
            tier,
            marketing_opt_in,
            total_spend,
            order_count,
            notes,
            rng,
        ),
    }
}

fn build_invalid_row(index: usize, rng: &mut DeterministicRng) -> RawCustomerRow {
    let locale = choose_locale(rng);
    let mut row = build_valid_row(200_000 + index, locale, rng);
    match index % 3 {
        0 => row.customer_id.clear(),
        1 => row.marketing_opt_in = "maybe".to_string(),
        _ => row.preferred_language = "fr-FR".to_string(),
    }
    row
}

fn build_us_row(
    customer_id: String,
    birth_date: NaiveDate,
    signup_date: NaiveDate,
    last_purchase_date: Option<NaiveDate>,
    status: &str,
    tier: &str,
    marketing_opt_in: &str,
    total_spend: f64,
    order_count: u32,
    notes: String,
    rng: &mut DeterministicRng,
) -> RawCustomerRow {
    let first = US_FIRST_NAMES[rng.range_usize(US_FIRST_NAMES.len())];
    let last = US_LAST_NAMES[rng.range_usize(US_LAST_NAMES.len())];
    let full_name = if rng.chance(18) {
        format!("{last}, {first}")
    } else {
        format!("{first} {last}")
    };
    let (city, region) = US_LOCATIONS[rng.range_usize(US_LOCATIONS.len())];
    let street = US_STREET_NAMES[rng.range_usize(US_STREET_NAMES.len())];
    let address_line = if rng.chance(35) {
        format!(
            "{}, Apt {}",
            format!(
                "{} {} {}",
                100 + rng.range_usize(9_800),
                street,
                street_suffix(rng)
            ),
            1 + rng.range_usize(28)
        )
    } else {
        format!(
            "{} {} {}",
            100 + rng.range_usize(9_800),
            street,
            street_suffix(rng)
        )
    };
    let postal_code = if rng.chance(40) {
        format!(
            "{:05}{:04}",
            10_000 + rng.range_usize(89_999),
            rng.range_usize(10_000)
        )
    } else {
        format!("{:05}", 10_000 + rng.range_usize(89_999))
    };
    let phone = match rng.range_usize(3) {
        0 => format!(
            "({}) {}-{:04}",
            200 + rng.range_usize(700),
            100 + rng.range_usize(900),
            rng.range_usize(10_000)
        ),
        1 => format!(
            "+1-{}-{}-{:04}",
            200 + rng.range_usize(700),
            100 + rng.range_usize(900),
            rng.range_usize(10_000)
        ),
        _ => format!(
            "{}{}{:04}",
            200 + rng.range_usize(700),
            100 + rng.range_usize(900),
            rng.range_usize(10_000)
        ),
    };

    RawCustomerRow {
        customer_id,
        full_name,
        email: format!("cust{:06}.us@example.test", 1 + rng.range_usize(900_000)),
        phone,
        address_line,
        city: city.to_string(),
        region: region.to_string(),
        postal_code,
        country: pick_country_alias(LocaleProfile::UsEnglish, rng).to_string(),
        birth_date: format_date_variant(birth_date, rng),
        signup_date: format_date_variant(signup_date, rng),
        last_purchase_date: last_purchase_date
            .map(|date| format_date_variant(date, rng))
            .unwrap_or_default(),
        status: status.to_string(),
        tier: tier.to_string(),
        preferred_language: pick_language_alias(LocaleProfile::UsEnglish, rng).to_string(),
        marketing_opt_in: marketing_opt_in.to_string(),
        total_spend: format_currency_variant(total_spend, rng),
        order_count: order_count.to_string(),
        notes,
    }
}

fn build_jp_row(
    customer_id: String,
    birth_date: NaiveDate,
    signup_date: NaiveDate,
    last_purchase_date: Option<NaiveDate>,
    status: &str,
    tier: &str,
    marketing_opt_in: &str,
    total_spend: f64,
    order_count: u32,
    notes: String,
    rng: &mut DeterministicRng,
) -> RawCustomerRow {
    let family = JP_FAMILY_NAMES[rng.range_usize(JP_FAMILY_NAMES.len())];
    let given = JP_GIVEN_NAMES[rng.range_usize(JP_GIVEN_NAMES.len())];
    let full_name = if rng.chance(55) {
        format!("{family}{given}")
    } else {
        format!("{family} {given}")
    };
    let (city, region, area) = JP_LOCATIONS[rng.range_usize(JP_LOCATIONS.len())];
    let address_line = if rng.chance(45) {
        format!(
            "{area}{}-{}-{}",
            1 + rng.range_usize(8),
            1 + rng.range_usize(20),
            1 + rng.range_usize(30)
        )
    } else {
        format!(
            "{area} {}-{}-{}",
            1 + rng.range_usize(8),
            1 + rng.range_usize(20),
            1 + rng.range_usize(30)
        )
    };
    let postal_digits = format!("{:07}", 1 + rng.range_usize(9_999_999));
    let postal_code = if rng.chance(50) {
        format!("〒{}", to_fullwidth_digits(&postal_digits))
    } else {
        postal_digits
    };
    let phone = match rng.range_usize(3) {
        0 => format!(
            "090-{:04}-{:04}",
            rng.range_usize(10_000),
            rng.range_usize(10_000)
        ),
        1 => format!(
            "+81 90 {:04} {:04}",
            rng.range_usize(10_000),
            rng.range_usize(10_000)
        ),
        _ => format!(
            "０９０-{:04}-{:04}",
            rng.range_usize(10_000),
            rng.range_usize(10_000)
        ),
    };

    RawCustomerRow {
        customer_id,
        full_name,
        email: format!("cust{:06}.jp@example.test", 1 + rng.range_usize(900_000)),
        phone,
        address_line,
        city: city.to_string(),
        region: region.to_string(),
        postal_code,
        country: pick_country_alias(LocaleProfile::Japanese, rng).to_string(),
        birth_date: format_date_variant(birth_date, rng),
        signup_date: format_date_variant(signup_date, rng),
        last_purchase_date: last_purchase_date
            .map(|date| format_date_variant(date, rng))
            .unwrap_or_default(),
        status: status.to_string(),
        tier: tier.to_string(),
        preferred_language: pick_language_alias(LocaleProfile::Japanese, rng).to_string(),
        marketing_opt_in: marketing_opt_in.to_string(),
        total_spend: format_currency_variant(total_spend, rng),
        order_count: order_count.to_string(),
        notes,
    }
}

fn build_in_row(
    customer_id: String,
    birth_date: NaiveDate,
    signup_date: NaiveDate,
    last_purchase_date: Option<NaiveDate>,
    status: &str,
    tier: &str,
    marketing_opt_in: &str,
    total_spend: f64,
    order_count: u32,
    notes: String,
    rng: &mut DeterministicRng,
) -> RawCustomerRow {
    let given = IN_GIVEN_NAMES[rng.range_usize(IN_GIVEN_NAMES.len())];
    let family = IN_FAMILY_NAMES[rng.range_usize(IN_FAMILY_NAMES.len())];
    let (city, region, road) = IN_LOCATIONS[rng.range_usize(IN_LOCATIONS.len())];
    let address_line = format!(
        "घर {}, {}, वार्ड {}",
        10 + rng.range_usize(800),
        road,
        1 + rng.range_usize(12)
    );
    let postal_code = format!("{:06}", 100_000 + rng.range_usize(899_999));
    let phone = match rng.range_usize(3) {
        0 => format!(
            "+91 {} {}",
            90_000 + rng.range_usize(10_000),
            10_000 + rng.range_usize(90_000)
        ),
        1 => format!(
            "0{}{}",
            900_000_000 + rng.range_usize(90_000_000),
            rng.range_usize(10)
        ),
        _ => format!(
            "{}{}",
            90_000 + rng.range_usize(10_000),
            100_000 + rng.range_usize(900_000)
        ),
    };

    RawCustomerRow {
        customer_id,
        full_name: format!("{given} {family}"),
        email: format!("cust{:06}.in@example.test", 1 + rng.range_usize(900_000)),
        phone,
        address_line,
        city: city.to_string(),
        region: region.to_string(),
        postal_code,
        country: pick_country_alias(LocaleProfile::Hindi, rng).to_string(),
        birth_date: format_date_variant(birth_date, rng),
        signup_date: format_date_variant(signup_date, rng),
        last_purchase_date: last_purchase_date
            .map(|date| format_date_variant(date, rng))
            .unwrap_or_default(),
        status: status.to_string(),
        tier: tier.to_string(),
        preferred_language: pick_language_alias(LocaleProfile::Hindi, rng).to_string(),
        marketing_opt_in: marketing_opt_in.to_string(),
        total_spend: format_currency_variant(total_spend, rng),
        order_count: order_count.to_string(),
        notes,
    }
}

fn build_cn_row(
    customer_id: String,
    birth_date: NaiveDate,
    signup_date: NaiveDate,
    last_purchase_date: Option<NaiveDate>,
    status: &str,
    tier: &str,
    marketing_opt_in: &str,
    total_spend: f64,
    order_count: u32,
    notes: String,
    rng: &mut DeterministicRng,
) -> RawCustomerRow {
    let family = CN_FAMILY_NAMES[rng.range_usize(CN_FAMILY_NAMES.len())];
    let given = CN_GIVEN_NAMES[rng.range_usize(CN_GIVEN_NAMES.len())];
    let full_name = if rng.chance(68) {
        format!("{family}{given}")
    } else {
        format!("{family} {given}")
    };
    let (city, region, road) = CN_LOCATIONS[rng.range_usize(CN_LOCATIONS.len())];
    let address_line = format!(
        "{}{}号 {}单元{}室",
        road,
        10 + rng.range_usize(180),
        1 + rng.range_usize(8),
        101 + rng.range_usize(700)
    );
    let postal_code = format!("{:06}", 100_000 + rng.range_usize(899_999));
    let phone = match rng.range_usize(3) {
        0 => format!(
            "+86 13{} {:04} {:04}",
            1 + rng.range_usize(8),
            rng.range_usize(10_000),
            rng.range_usize(10_000)
        ),
        1 => format!(
            "13{}{}{:04}",
            1 + rng.range_usize(8),
            1_000 + rng.range_usize(9_000),
            rng.range_usize(10_000)
        ),
        _ => format!(
            "＋86 13{} {:04} {:04}",
            1 + rng.range_usize(8),
            rng.range_usize(10_000),
            rng.range_usize(10_000)
        ),
    };

    RawCustomerRow {
        customer_id,
        full_name,
        email: format!("cust{:06}.cn@example.test", 1 + rng.range_usize(900_000)),
        phone,
        address_line,
        city: city.to_string(),
        region: region.to_string(),
        postal_code,
        country: pick_country_alias(LocaleProfile::Chinese, rng).to_string(),
        birth_date: format_date_variant(birth_date, rng),
        signup_date: format_date_variant(signup_date, rng),
        last_purchase_date: last_purchase_date
            .map(|date| format_date_variant(date, rng))
            .unwrap_or_default(),
        status: status.to_string(),
        tier: tier.to_string(),
        preferred_language: pick_language_alias(LocaleProfile::Chinese, rng).to_string(),
        marketing_opt_in: marketing_opt_in.to_string(),
        total_spend: format_currency_variant(total_spend, rng),
        order_count: order_count.to_string(),
        notes,
    }
}

fn derive_order_count(status: &str, rng: &mut DeterministicRng) -> u32 {
    match status.to_ascii_lowercase().as_str() {
        "active" => 3 + rng.range_u32(18),
        "inactive" => rng.range_u32(8),
        "pending" => rng.range_u32(4),
        "banned" => rng.range_u32(6),
        _ => rng.range_u32(5),
    }
}

fn derive_total_spend(order_count: u32, tier: &str, rng: &mut DeterministicRng) -> f64 {
    let tier_multiplier = match tier.to_ascii_lowercase().as_str() {
        "bronze" => 0.9,
        "silv" | "silver" => 1.1,
        "gold" => 1.35,
        "platinum" => 1.7,
        _ => 1.0,
    };
    let base = order_count as f64 * (35.0 + rng.range_u32(110) as f64);
    (base * tier_multiplier) + rng.range_u32(80) as f64
}

fn derive_last_purchase_date(
    signup_date: NaiveDate,
    order_count: u32,
    rng: &mut DeterministicRng,
) -> Option<NaiveDate> {
    if order_count == 0 && rng.chance(70) {
        return None;
    }
    Some(date_from_offset(signup_date, 30 + rng.range_i64(780)))
}

fn street_suffix(rng: &mut DeterministicRng) -> &'static str {
    match rng.range_usize(4) {
        0 => "Street",
        1 => "Avenue",
        2 => "Road",
        _ => "Lane",
    }
}

fn pick_country_alias(locale: LocaleProfile, rng: &mut DeterministicRng) -> &'static str {
    match locale {
        LocaleProfile::UsEnglish => ["United States", "US", "usa"][rng.range_usize(3)],
        LocaleProfile::Japanese => ["Japan", "jp", "JPN"][rng.range_usize(3)],
        LocaleProfile::Hindi => ["India", "IN", "ind"][rng.range_usize(3)],
        LocaleProfile::Chinese => ["China", "CN", "PRC"][rng.range_usize(3)],
    }
}

fn pick_language_alias(locale: LocaleProfile, rng: &mut DeterministicRng) -> &'static str {
    match locale {
        LocaleProfile::UsEnglish => ["en", "EN-US", "en_us"][rng.range_usize(3)],
        LocaleProfile::Japanese => ["ja", "jp", "JA"][rng.range_usize(3)],
        LocaleProfile::Hindi => ["hi", "HI-IN", "hi_in"][rng.range_usize(3)],
        LocaleProfile::Chinese => ["zh", "zh-CN", "zh_cn"][rng.range_usize(3)],
    }
}

fn format_currency_variant(amount: f64, rng: &mut DeterministicRng) -> String {
    match rng.range_usize(4) {
        0 => format!("{amount:.2}"),
        1 => format!("${amount:.2}"),
        2 => format!("{:.2}", amount).replace('.', "."),
        _ => format!("{amount:.2}"),
    }
}

fn format_date_variant(date: NaiveDate, rng: &mut DeterministicRng) -> String {
    match rng.range_usize(4) {
        0 => date.format("%Y-%m-%d").to_string(),
        1 => date.format("%Y/%m/%d").to_string(),
        2 => date.format("%d-%m-%Y").to_string(),
        _ => date.format("%m/%d/%Y").to_string(),
    }
}

fn date_from_offset(base: NaiveDate, offset_days: i64) -> NaiveDate {
    base + Duration::days(offset_days)
}

fn to_fullwidth_digits(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            '0' => '０',
            '1' => '１',
            '2' => '２',
            '3' => '３',
            '4' => '４',
            '5' => '５',
            '6' => '６',
            '7' => '７',
            '8' => '８',
            '9' => '９',
            other => other,
        })
        .collect()
}

#[derive(Debug, Clone)]
struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        let seed = if seed == 0 {
            0x9e37_79b9_7f4a_7c15
        } else {
            seed
        };
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 7;
        self.state ^= self.state >> 9;
        self.state ^= self.state << 8;
        self.state
    }

    fn range_usize(&mut self, upper: usize) -> usize {
        if upper <= 1 {
            0
        } else {
            (self.next_u64() % upper as u64) as usize
        }
    }

    fn range_u32(&mut self, upper: u32) -> u32 {
        if upper <= 1 {
            0
        } else {
            (self.next_u64() % upper as u64) as u32
        }
    }

    fn range_i64(&mut self, upper: i64) -> i64 {
        if upper <= 1 {
            0
        } else {
            (self.next_u64() % upper as u64) as i64
        }
    }

    fn chance(&mut self, percent: usize) -> bool {
        self.range_usize(100) < percent
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{config::GenerateRawSampleConfig, format_dataset};

    use super::generate_raw_sample;

    #[test]
    fn generated_raw_sample_yields_expected_formatted_count() {
        let temp_dir = std::env::temp_dir().join("decision-pack-customers-etl-sample-test");
        let output_raw = temp_dir.join("raw_customers_test.csv");
        let config = GenerateRawSampleConfig {
            output_raw: output_raw.clone(),
            target_formatted_count: 12,
            invalid_row_count: 3,
            seed: 42,
        };

        let summary = generate_raw_sample(&config).expect("sample generation should succeed");
        let input = fs::read_to_string(&output_raw).expect("generated raw file should exist");
        let run = format_dataset(&input);

        assert_eq!(summary.target_formatted_count, 12);
        assert_eq!(run.report.rows_written, 15);
        assert!(run.report.rows_with_failures >= 3);

        let _ = fs::remove_file(&output_raw);
        let _ = fs::remove_file(temp_dir.join("raw_customers_test_metadata.json"));
    }
}
