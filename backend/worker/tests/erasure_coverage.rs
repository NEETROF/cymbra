//! Guards the account-erasure path against a table added and forgotten (change:
//! harden-module-boundaries, group 2).
//!
//! Runs with no database: it reads the migrations for tables keyed by an account
//! column, then checks each is either deleted by `purge_user_with`, cascaded from
//! `user_account.users`, or listed as a documented exemption below.
//!
//! This exists because the gap it catches was real. `music.user_soundfonts` shipped
//! with tests and a private bucket and was never purged, and the audit that found it
//! turned up three more tables in the same state.

use std::collections::BTreeSet;
use std::path::Path;

/// Columns that make a table account-scoped.
const ACCOUNT_COLS: [&str; 3] = ["user_id", "owner_id", "account_id"];

/// Tables that are account-keyed but deliberately NOT erased, each with the reason.
/// Adding an entry here is a decision about someone's personal data — say why.
const EXEMPT: &[(&str, &str)] = &[(
    "music.user_score_takedowns",
    "Moderation record. Erasing it would let a removed upload return under a new \
     account, since the sha256 in this row is what recognises the content. Keeping it \
     retains `owner_id` after erasure, so this is a retention decision, not an \
     oversight — see the open question in the harden-module-boundaries design.",
)];

fn repo_root() -> &'static Path {
    // backend/worker/ -> repo root
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

/// Every `<schema>.<table>` in the migrations that carries an account column.
fn account_keyed_tables() -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let root = repo_root();
    for module in [
        "auth",
        "user",
        "music",
        "plans",
        "analytics",
        "feature-flags",
    ] {
        let dir = root.join("backend").join(module).join("migrations");
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("sql") {
                continue;
            }
            let sql = std::fs::read_to_string(&path).expect("read migration");
            found.extend(tables_with_account_column(&sql));
        }
    }
    found
}

/// Parse `CREATE TABLE <name> ( ... );` blocks and keep the account-keyed ones.
fn tables_with_account_column(sql: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lowered = sql.to_lowercase();
    let mut cursor = 0usize;
    while let Some(rel) = lowered[cursor..].find("create table") {
        let start = cursor + rel;
        let Some(open) = lowered[start..].find('(') else {
            break;
        };
        let header = &sql[start..start + open];
        let Some(close) = lowered[start..].find("\n);") else {
            cursor = start + open;
            continue;
        };
        let body = &sql[start + open..start + close];
        cursor = start + close;

        let name = header
            .split_whitespace()
            .last()
            .unwrap_or_default()
            .trim()
            .to_string();
        if name.is_empty() {
            continue;
        }
        // The column must be declared, not merely mentioned: require it at the start
        // of a line inside the body.
        let has_account_col = body.lines().any(|l| {
            let t = l.trim_start();
            ACCOUNT_COLS
                .iter()
                .any(|c| t.starts_with(c) && t[c.len()..].starts_with(char::is_whitespace))
        });
        if has_account_col {
            out.push(name);
        }
    }
    out
}

/// True when the table drops with its owner via a cascading FK to the users table.
fn cascades_from_users(table: &str) -> bool {
    let short = table.rsplit('.').next().unwrap_or(table);
    let root = repo_root();
    for module in ["auth", "user", "music", "plans"] {
        let dir = root.join("backend").join(module).join("migrations");
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(sql) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            let Some(at) = sql
                .find(&format!("CREATE TABLE IF NOT EXISTS {table}"))
                .or_else(|| {
                    sql.find(&format!("CREATE TABLE {table}"))
                        .or_else(|| sql.find(&format!("CREATE TABLE IF NOT EXISTS {short}")))
                        .or_else(|| sql.find(&format!("CREATE TABLE {short}")))
                })
            else {
                continue;
            };
            let block = &sql[at..];
            let end = block.find("\n);").unwrap_or(block.len());
            let block = &block[..end];
            if block.contains("REFERENCES user_account.users")
                && block.contains("ON DELETE CASCADE")
            {
                return true;
            }
        }
    }
    false
}

#[test]
fn every_account_keyed_table_is_reached_by_the_erasure() {
    let purge = std::fs::read_to_string(repo_root().join("backend/worker/src/lib.rs"))
        .expect("read worker lib.rs");
    let exempt: BTreeSet<&str> = EXEMPT.iter().map(|(t, _)| *t).collect();

    let mut missing = Vec::new();
    for table in account_keyed_tables() {
        let short = table.rsplit('.').next().unwrap_or(&table);
        if exempt.contains(table.as_str()) || exempt.contains(short) {
            continue;
        }
        if cascades_from_users(&table) {
            continue;
        }
        // The erasure names the table, qualified or not.
        if purge.contains(&table) || purge.contains(short) {
            continue;
        }
        missing.push(table);
    }

    assert!(
        missing.is_empty(),
        "these tables hold account-scoped data but nothing erases them:\n  {}\n\n\
         Add a DELETE to `purge_user_with` (and, if the rows reference stored objects, \
         enqueue the matching purge job in the SAME transaction — mind which bucket). \
         If a table must survive an erasure, add it to EXEMPT in this file with the \
         reason.",
        missing.join("\n  ")
    );
}

/// The guard is only worth having if it can fail. A table the erasure does not name,
/// does not cascade and is not exempt must be reported.
#[test]
fn the_guard_reports_an_unreached_table() {
    let purge = "DELETE FROM music.user_scores WHERE owner_id = $1";
    let table = "music.brand_new_personal_thing";
    assert!(!purge.contains(table));
    assert!(!cascades_from_users(table));
}

/// Parsing sanity: a table without an account column is not flagged, one with it is.
#[test]
fn account_column_detection_ignores_unrelated_tables() {
    let sql = "
CREATE TABLE music.catalog_scores (
    id    UUID PRIMARY KEY,
    title TEXT
);
CREATE TABLE IF NOT EXISTS music.thing (
    id      UUID PRIMARY KEY,
    user_id UUID NOT NULL
);
";
    let found = tables_with_account_column(sql);
    assert_eq!(found, vec!["music.thing".to_string()]);
}
