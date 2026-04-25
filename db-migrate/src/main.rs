use sqlx::{Executor, PgPool, postgres::PgPoolOptions};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct Config {
    database_url: String,
    migrations_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Migration {
    version: String,
    name: String,
    path: PathBuf,
    sql: String,
    checksum: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse_from_env()
        .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message))?;
    let migrations = load_migrations(&config.migrations_dir)?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&config.database_url)
        .await?;

    ensure_migrations_table(&pool).await?;

    if migrations.is_empty() {
        println!(
            "migration file was not found: {}",
            config.migrations_dir.display()
        );
        return Ok(());
    }

    let mut applied_count = 0usize;
    let mut skipped_count = 0usize;
    for migration in migrations {
        match apply_migration(&pool, &migration).await? {
            ApplyOutcome::Applied => {
                applied_count += 1;
                println!("applied migration: {}", migration.name);
            }
            ApplyOutcome::Skipped => {
                skipped_count += 1;
                println!("skipped migration: {}", migration.name);
            }
        }
    }

    println!(
        "migration run completed: applied={}, skipped={}",
        applied_count, skipped_count
    );
    Ok(())
}

impl Config {
    fn parse_from_env() -> Result<Self, String> {
        let mut database_url = None;
        let mut migrations_dir = None;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--database-url" => {
                    database_url = Some(next_string(&mut args, "--database-url")?);
                }
                "--migrations-dir" => {
                    migrations_dir =
                        Some(PathBuf::from(next_string(&mut args, "--migrations-dir")?));
                }
                "--help" | "-h" => return Err(usage()),
                other => return Err(format!("unknown argument: `{other}`\n\n{}", usage())),
            }
        }

        let database_url = database_url
            .or_else(|| env::var("DATABASE_URL").ok())
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| format!("--database-url or DATABASE_URL is required\n\n{}", usage()))?;

        Ok(Self {
            database_url,
            migrations_dir: migrations_dir.unwrap_or_else(|| PathBuf::from("db/migrations")),
        })
    }
}

fn next_string(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{flag} is missing a value"))
}

fn usage() -> String {
    "usage: cargo run -p db-migrate -- [--database-url <URL>] [--migrations-dir <DIR>]".to_string()
}

fn load_migrations(dir: &Path) -> Result<Vec<Migration>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("sql") {
            files.push(path);
        }
    }
    files.sort_by(|left, right| left.file_name().cmp(&right.file_name()));

    files.into_iter().map(read_migration).collect()
}

fn read_migration(path: PathBuf) -> Result<Migration, Box<dyn std::error::Error>> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("invalid migration file name: {}", path.display()))?
        .to_string();
    let version = parse_version(&name)?;
    let sql = fs::read_to_string(&path)?;
    let checksum = checksum_hex(&sql);

    Ok(Migration {
        version,
        name,
        path,
        sql,
        checksum,
    })
}

fn parse_version(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let Some((version, _)) = name.split_once('_') else {
        return Err(format!("migration file must start with <version>_: {name}").into());
    };
    if version.is_empty() || !version.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(format!("migration version must be numeric: {name}").into());
    }
    Ok(version.to_string())
}

async fn ensure_migrations_table(pool: &PgPool) -> Result<(), sqlx::Error> {
    pool.execute(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            checksum TEXT NOT NULL,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .await?;
    Ok(())
}

enum ApplyOutcome {
    Applied,
    Skipped,
}

async fn apply_migration(
    pool: &PgPool,
    migration: &Migration,
) -> Result<ApplyOutcome, Box<dyn std::error::Error>> {
    let existing_checksum = sqlx::query_scalar::<_, String>(
        "SELECT checksum FROM schema_migrations WHERE version = $1",
    )
    .bind(&migration.version)
    .fetch_optional(pool)
    .await?;

    if let Some(existing_checksum) = existing_checksum {
        if existing_checksum != migration.checksum {
            return Err(format!(
                "migration checksum mismatch for {}: database={}, file={}",
                migration.name, existing_checksum, migration.checksum
            )
            .into());
        }
        return Ok(ApplyOutcome::Skipped);
    }

    let mut tx = pool.begin().await?;
    for statement in split_sql_statements(&migration.sql) {
        tx.execute(statement.as_str()).await?;
    }
    sqlx::query(
        r#"
        INSERT INTO schema_migrations (version, name, checksum)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(&migration.version)
    .bind(&migration.name)
    .bind(&migration.checksum)
    .execute(tx.as_mut())
    .await?;
    tx.commit().await?;

    Ok(ApplyOutcome::Applied)
}

fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut chars = sql.chars().peekable();
    let mut in_single_quote = false;
    let mut in_line_comment = false;

    while let Some(ch) = chars.next() {
        if in_line_comment {
            current.push(ch);
            if ch == '\n' {
                in_line_comment = false;
            }
            continue;
        }

        if !in_single_quote && ch == '-' && chars.peek() == Some(&'-') {
            current.push(ch);
            current.push(chars.next().unwrap());
            in_line_comment = true;
            continue;
        }

        if ch == '\'' {
            current.push(ch);
            if in_single_quote && chars.peek() == Some(&'\'') {
                current.push(chars.next().unwrap());
            } else {
                in_single_quote = !in_single_quote;
            }
            continue;
        }

        if ch == ';' && !in_single_quote {
            let statement = current.trim();
            if !statement.is_empty() {
                statements.push(statement.to_string());
            }
            current.clear();
            continue;
        }

        current.push(ch);
    }

    let statement = current.trim();
    if !statement.is_empty() {
        statements.push(statement.to_string());
    }

    statements
}

fn checksum_hex(input: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::{checksum_hex, parse_version, split_sql_statements};

    #[test]
    fn parses_numeric_migration_version() {
        assert_eq!(
            parse_version("202604172120_initial_schema.sql").unwrap(),
            "202604172120"
        );
    }

    #[test]
    fn rejects_migration_name_without_numeric_prefix() {
        assert!(parse_version("initial_schema.sql").is_err());
        assert!(parse_version("abc_initial_schema.sql").is_err());
    }

    #[test]
    fn splits_sql_statements_without_splitting_quoted_semicolons() {
        let sql = "CREATE TABLE t (v TEXT DEFAULT 'a;b');\n-- comment;\nCREATE INDEX i ON t (v);";
        let statements = split_sql_statements(sql);

        assert_eq!(statements.len(), 2);
        assert!(statements[0].contains("'a;b'"));
        assert!(statements[1].contains("CREATE INDEX"));
    }

    #[test]
    fn checksum_is_stable_for_same_input() {
        assert_eq!(checksum_hex("select 1"), checksum_hex("select 1"));
        assert_ne!(checksum_hex("select 1"), checksum_hex("select 2"));
    }
}
