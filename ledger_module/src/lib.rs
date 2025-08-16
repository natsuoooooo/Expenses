use rusqlite::{Connection, params};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Kind {
    Expense,
    Income,
}
impl Kind {
    pub fn to_i64(self) -> i64 {
        match self {
            Kind::Expense => 0,
            Kind::Income => 1,
        }
    }
    pub fn from_i64(v: i64) -> Self {
        if v == 0 { Kind::Expense } else { Kind::Income }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub id: i64,
    pub kind: Kind,
    pub amount: i64,
    pub category: String,
    pub note: Option<String>,
    pub created_at: String,
}

pub fn open_db() -> rusqlite::Result<Connection> {
    Connection::open("ledger.db")
}

pub fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS entries (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            kind       INTEGER NOT NULL CHECK(kind IN (0, 1)),
            amount     INTEGER NOT NULL CHECK(amount > 0),
            category   TEXT NOT NULL,
            note       TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        "#,
    )
}

pub fn add_entry(
    conn: &Connection,
    kind: Kind,
    amount: i64,
    category: &str,
    note: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO entries (kind, amount, category, note) VALUES (?1, ?2, ?3, ?4)",
        params![kind.to_i64(), amount, category, note],
    )?;
    Ok(())
}

pub fn list_entries(conn: &Connection) -> rusqlite::Result<Vec<Entry>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, kind, amount, category, note, created_at
        FROM entries
        ORDER BY datetime(created_at) DESC
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Entry {
            id: row.get(0)?,
            kind: Kind::from_i64(row.get::<_, i64>(1)?),
            amount: row.get(2)?,
            category: row.get(3)?,
            note: row.get::<_, Option<String>>(4)?,
            created_at: row.get(5)?,
        })
    })?;

    let mut v = Vec::new();
    for r in rows {
        v.push(r?);
    }
    Ok(v)
}

pub fn delete_entry(conn: &Connection, id: i64) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM entries WHERE id = ?1", params![id])
}

pub struct MonthSummary {
    pub month: String,
    pub expense: i64,
    pub income: i64,
    pub balance: i64,
}

pub fn month_summary(conn: &Connection, ym: &str) -> rusqlite::Result<MonthSummary> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            SUM(CASE WHEN kind = 1 THEN amount ELSE 0 END) AS income,
            SUM(CASE WHEN kind = 0 THEN amount ELSE 0 END) AS expense
        FROM entries
        WHERE substr(created_at, 1, 7) = ?1
        "#,
    )?;

    let (income_opt, expense_opt): (Option<i64>, Option<i64>) =
        stmt.query_row(params![ym], |row| Ok((row.get(0)?, row.get(1)?)))?;

    let income = income_opt.unwrap_or(0);
    let expense = expense_opt.unwrap_or(0);

    Ok(MonthSummary {
        month: ym.to_string(),
        income,
        expense,
        balance: income - expense,
    })
}

#[derive(Debug)]
pub struct CategoryTotal {
    pub category: String,
    pub total: i64,
}

pub fn category_totals_by_kind(
    conn: &Connection,
    ym: &str,
    kind: Kind, // Kind::Expense or Kind::Income
) -> rusqlite::Result<Vec<CategoryTotal>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT category, SUM(amount) AS total
        FROM entries
        WHERE kind = ?2 AND substr(created_at, 1, 7) = ?1
        GROUP BY category
        ORDER BY total DESC, category ASC
        "#,
    )?;

    let rows = stmt.query_map(params![ym, kind.to_i64()], |row| {
        Ok(CategoryTotal {
            category: row.get(0)?,
            total: row.get(1)?,
        })
    })?;

    let mut v = Vec::new();
    for r in rows {
        v.push(r?);
    }
    Ok(v)
}
