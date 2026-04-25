//! `customers-etl` の PostgreSQL 永続化です。

use crate::config::FormatConfig;
use crate::formatter::FormatRun;
use crate::schema::Column;
use chrono::NaiveDate;
use sqlx::{Postgres, Transaction, postgres::PgPoolOptions};

#[derive(Debug, Clone, Copy)]
pub struct PersistSummary {
    pub customers_upserted: usize,
    pub issues_inserted: usize,
    pub rows_skipped_for_persist: usize,
}

#[derive(Debug)]
struct CustomerRecord {
    customer_id: String,
    full_name: String,
    email: Option<String>,
    phone: Option<String>,
    address_line: Option<String>,
    city: Option<String>,
    region: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    birth_date: Option<NaiveDate>,
    signup_date: Option<NaiveDate>,
    last_purchase_date: Option<NaiveDate>,
    status: Option<String>,
    tier: Option<String>,
    preferred_language: Option<String>,
    marketing_opt_in: Option<bool>,
    total_spend: Option<f64>,
    order_count: Option<i32>,
    notes: Option<String>,
}

pub async fn persist_run(
    config: &FormatConfig,
    run: &FormatRun,
) -> Result<PersistSummary, Box<dyn std::error::Error>> {
    let database_url = config
        .database_url
        .as_deref()
        .ok_or_else(|| "database_url が未設定です".to_string())?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    let mut tx = pool.begin().await?;

    let mut customers_upserted = 0usize;
    let mut rows_skipped_for_persist = 0usize;
    for row in run.cleaned_rows.iter().skip(1) {
        match CustomerRecord::from_row(row) {
            Ok(customer) => {
                upsert_customer(&mut tx, customer).await?;
                customers_upserted += 1;
            }
            Err(_) => {
                rows_skipped_for_persist += 1;
            }
        }
    }

    let mut issues_inserted = 0usize;
    for issue in &run.report.issues {
        insert_issue(&mut tx, config, issue).await?;
        issues_inserted += 1;
    }

    upsert_job_run(&mut tx, config, issues_inserted).await?;
    tx.commit().await?;

    Ok(PersistSummary {
        customers_upserted,
        issues_inserted,
        rows_skipped_for_persist,
    })
}

impl CustomerRecord {
    fn from_row(row: &[String]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            customer_id: required(row, Column::CustomerId)?,
            full_name: required(row, Column::FullName)?,
            email: optional(row, Column::Email),
            phone: optional(row, Column::Phone),
            address_line: optional(row, Column::AddressLine),
            city: optional(row, Column::City),
            region: optional(row, Column::Region),
            postal_code: optional(row, Column::PostalCode),
            country: optional(row, Column::Country),
            birth_date: optional_date(row, Column::BirthDate),
            signup_date: optional_date(row, Column::SignupDate),
            last_purchase_date: optional_date(row, Column::LastPurchaseDate),
            status: optional(row, Column::Status),
            tier: optional(row, Column::Tier),
            preferred_language: optional(row, Column::PreferredLanguage),
            marketing_opt_in: optional(row, Column::MarketingOptIn)
                .map(|value| value.eq_ignore_ascii_case("true")),
            total_spend: optional(row, Column::TotalSpend)
                .map(|value| value.parse())
                .transpose()?,
            order_count: optional(row, Column::OrderCount)
                .map(|value| value.parse())
                .transpose()?,
            notes: optional(row, Column::Notes),
        })
    }
}

async fn upsert_customer(
    tx: &mut Transaction<'_, Postgres>,
    customer: CustomerRecord,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO customers (
            customer_id,
            full_name,
            email,
            phone,
            address_line,
            city,
            region,
            postal_code,
            country,
            birth_date,
            signup_date,
            last_purchase_date,
            status,
            tier,
            preferred_language,
            marketing_opt_in,
            total_spend,
            order_count,
            notes
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9,
            $10, $11, $12, $13, $14, $15, $16, $17, $18, $19
        )
        ON CONFLICT (customer_id) DO UPDATE
        SET
            full_name = EXCLUDED.full_name,
            email = EXCLUDED.email,
            phone = EXCLUDED.phone,
            address_line = EXCLUDED.address_line,
            city = EXCLUDED.city,
            region = EXCLUDED.region,
            postal_code = EXCLUDED.postal_code,
            country = EXCLUDED.country,
            birth_date = EXCLUDED.birth_date,
            signup_date = EXCLUDED.signup_date,
            last_purchase_date = EXCLUDED.last_purchase_date,
            status = EXCLUDED.status,
            tier = EXCLUDED.tier,
            preferred_language = EXCLUDED.preferred_language,
            marketing_opt_in = EXCLUDED.marketing_opt_in,
            total_spend = EXCLUDED.total_spend,
            order_count = EXCLUDED.order_count,
            notes = EXCLUDED.notes,
            updated_at = NOW()
        "#,
    )
    .bind(customer.customer_id)
    .bind(customer.full_name)
    .bind(customer.email)
    .bind(customer.phone)
    .bind(customer.address_line)
    .bind(customer.city)
    .bind(customer.region)
    .bind(customer.postal_code)
    .bind(customer.country)
    .bind(customer.birth_date)
    .bind(customer.signup_date)
    .bind(customer.last_purchase_date)
    .bind(customer.status)
    .bind(customer.tier)
    .bind(customer.preferred_language)
    .bind(customer.marketing_opt_in)
    .bind(customer.total_spend)
    .bind(customer.order_count)
    .bind(customer.notes)
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

async fn insert_issue(
    tx: &mut Transaction<'_, Postgres>,
    config: &FormatConfig,
    issue: &crate::report::IssueRecord,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO customer_load_issues (
            customer_id,
            column_name,
            issue_code,
            raw_value,
            message,
            source_row_number,
            run_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(Option::<String>::None)
    .bind(
        issue
            .column
            .map(|column| column.header().to_string())
            .unwrap_or_else(|| "__row__".to_string()),
    )
    .bind(issue.kind.as_str())
    .bind(issue.raw_value.clone())
    .bind(issue.reason.clone())
    .bind(issue.line_number as i64)
    .bind(config.run_id.clone())
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

async fn upsert_job_run(
    tx: &mut Transaction<'_, Postgres>,
    config: &FormatConfig,
    issues_inserted: usize,
) -> Result<(), sqlx::Error> {
    let artifact_uri = config.output_dir.display().to_string();
    let error_message = if issues_inserted == 0 {
        None
    } else {
        Some(format!("completed with {issues_inserted} issues"))
    };

    sqlx::query(
        r#"
        INSERT INTO etl_job_runs (
            job_id,
            job_kind,
            status,
            requested_at,
            started_at,
            completed_at,
            source_uri,
            artifact_uri,
            error_message
        ) VALUES ($1, 'customers-etl', 'succeeded', NOW(), NOW(), NOW(), $2, $3, $4)
        ON CONFLICT (job_id) DO UPDATE
        SET
            job_kind = EXCLUDED.job_kind,
            status = EXCLUDED.status,
            source_uri = EXCLUDED.source_uri,
            artifact_uri = EXCLUDED.artifact_uri,
            error_message = EXCLUDED.error_message,
            completed_at = NOW()
        "#,
    )
    .bind(config.run_id.clone())
    .bind(config.input.display().to_string())
    .bind(artifact_uri)
    .bind(error_message)
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

fn required(row: &[String], column: Column) -> Result<String, Box<dyn std::error::Error>> {
    optional(row, column).ok_or_else(|| format!("{} が空です", column.header()).into())
}

fn optional(row: &[String], column: Column) -> Option<String> {
    row.get(column.index())
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn optional_date(row: &[String], column: Column) -> Option<NaiveDate> {
    optional(row, column).and_then(|value| NaiveDate::parse_from_str(&value, "%Y-%m-%d").ok())
}

#[cfg(test)]
mod tests {
    use super::{CustomerRecord, optional_date};
    use crate::schema::{Column, HEADER_ROW};
    use chrono::NaiveDate;

    fn valid_row() -> Vec<String> {
        vec![
            "CUST1".to_string(),
            "Alice Smith".to_string(),
            "alice@example.com".to_string(),
            "09011112222".to_string(),
            "1 Main St".to_string(),
            "Tokyo".to_string(),
            "Tokyo".to_string(),
            "1000001".to_string(),
            "Japan".to_string(),
            "1980-01-01".to_string(),
            "2020-01-01".to_string(),
            "2024-01-01".to_string(),
            "active".to_string(),
            "gold".to_string(),
            "ja".to_string(),
            "true".to_string(),
            "1200.50".to_string(),
            "3".to_string(),
            "note".to_string(),
        ]
    }

    #[test]
    fn optional_date_parses_normalized_date() {
        let row = valid_row();

        assert_eq!(
            optional_date(&row, Column::BirthDate),
            NaiveDate::from_ymd_opt(1980, 1, 1)
        );
    }

    #[test]
    fn optional_date_treats_invalid_optional_date_as_null() {
        let mut row = valid_row();
        row[Column::BirthDate.index()] = "1979-11-31".to_string();

        assert_eq!(optional_date(&row, Column::BirthDate), None);
    }

    #[test]
    fn customer_record_accepts_invalid_optional_dates_as_nulls() {
        let mut row = valid_row();
        row[Column::BirthDate.index()] = "1979-11-31".to_string();

        let record = CustomerRecord::from_row(&row).unwrap();

        assert_eq!(record.birth_date, None);
        assert_eq!(record.signup_date, NaiveDate::from_ymd_opt(2020, 1, 1));
    }

    #[test]
    fn fixture_row_has_expected_width() {
        assert_eq!(valid_row().len(), HEADER_ROW.len());
    }
}
