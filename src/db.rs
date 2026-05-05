use anyhow::Result;
use libsql::params;
use tracing::info;

const MIGRATIONS: &[(&str, &str)] = &[
    ("001_initial", include_str!("../migrations/001_initial.sql")),
    ("002_loser_counts", include_str!("../migrations/002_loser_counts.sql")),
];

pub async fn connect(url: &str, auth_token: &str) -> Result<libsql::Database> {
    let db = libsql::Builder::new_remote(url.to_string(), auth_token.to_string())
        .build()
        .await?;
    Ok(db)
}

pub async fn run_migrations(db: &libsql::Database) -> Result<()> {
    let conn = db.connect()?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version    TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .await?;

    for (version, sql) in MIGRATIONS {
        let mut rows = conn
            .query(
                "SELECT 1 FROM schema_migrations WHERE version = ?1",
                params![*version],
            )
            .await?;

        if rows.next().await?.is_none() {
            conn.execute_batch(sql).await?;
            conn.execute(
                "INSERT INTO schema_migrations (version) VALUES (?1)",
                params![*version],
            )
            .await?;
            info!("Applied migration: {}", version);
        }
    }

    Ok(())
}

pub async fn increment_count(db: &libsql::Database, user_id: &str) -> Result<()> {
    let conn = db.connect()?;
    conn.execute(
        "INSERT INTO ign_counts (user_id, count, updated_at)
         VALUES (?1, 1, datetime('now'))
         ON CONFLICT(user_id) DO UPDATE SET
             count = count + 1,
             updated_at = datetime('now')",
        params![user_id],
    )
    .await?;
    Ok(())
}

pub async fn increment_loser_count(db: &libsql::Database, user_id: &str) -> Result<()> {
    let conn = db.connect()?;
    conn.execute(
        "INSERT INTO loser_counts (user_id, count, updated_at)
         VALUES (?1, 1, datetime('now'))
         ON CONFLICT(user_id) DO UPDATE SET
             count = count + 1,
             updated_at = datetime('now')",
        params![user_id],
    )
    .await?;
    Ok(())
}
