//! 列ごとの整形規則です。
//!
//! 各モジュールが 1 列ぶんの正規化・検証方針を持つため、ライブラリ構造は
//! CSV スキーマと 1 対 1 で対応します。下のディスパッチャだけが、
//! スキーマ列と具体的な処理器の対応付けを担います。

pub mod address_line;
pub mod birth_date;
pub mod city;
pub mod country;
pub mod customer_id;
pub mod email;
pub mod full_name;
pub mod last_purchase_date;
pub mod marketing_opt_in;
pub mod notes;
pub mod order_count;
pub mod phone;
pub mod postal_code;
pub mod preferred_language;
pub mod region;
pub mod signup_date;
pub mod status;
pub mod tier;
pub mod total_spend;

use crate::report::FieldResult;
use crate::schema::Column;

/// 生のフィールド値を、対応列の規則を持つ整形器へ振り分けます。
pub fn process(column: Column, raw: &str) -> FieldResult {
    match column {
        Column::CustomerId => customer_id::process(raw),
        Column::FullName => full_name::process(raw),
        Column::Email => email::process(raw),
        Column::Phone => phone::process(raw),
        Column::AddressLine => address_line::process(raw),
        Column::City => city::process(raw),
        Column::Region => region::process(raw),
        Column::PostalCode => postal_code::process(raw),
        Column::Country => country::process(raw),
        Column::BirthDate => birth_date::process(raw),
        Column::SignupDate => signup_date::process(raw),
        Column::LastPurchaseDate => last_purchase_date::process(raw),
        Column::Status => status::process(raw),
        Column::Tier => tier::process(raw),
        Column::PreferredLanguage => preferred_language::process(raw),
        Column::MarketingOptIn => marketing_opt_in::process(raw),
        Column::TotalSpend => total_spend::process(raw),
        Column::OrderCount => order_count::process(raw),
        Column::Notes => notes::process(raw),
    }
}
