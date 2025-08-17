# Expense Tracker (Rust)

A simple expense & income tracker written in Rust.  
Uses SQLite for storage, with a modular design:

- `cli/` : CLI app (user interface)
- `ledger_module/` : Core module (DB access & domain logic)

---

## Features
- Add expenses and income
- List all entries
- Delete by ID
- Monthly summary (income / expense / balance)
- Category totals (per month, for expense / income / both)
- Range summary across months
- Export entries to CSV (UTF-8 with BOM for Excel compatibility)

---

## CLI Quick Start

### Add entry
```bash
### Expense
cargo run -p cli -- add expense 1200 food Lunch

### Income
cargo run -p cli -- add income 50000 salary "August salary"
```

### List entries
```bash
cargo run -p cli -- list
```

### Delete entry
```bash
cargo run -p cli -- delete <id>
```

### Monthly summary
```bash
# Current month
cargo run -p cli -- report month

# Specific month
cargo run -p cli -- report month 2025-08
```

### Category report
```bash
# Expense only (default)
cargo run -p cli -- report category

# Income only
cargo run -p cli -- report category --income

# Both
cargo run -p cli -- report category --both

# For a specific month
cargo run -p cli -- report category 2025-08 --both
```

### Range summary
```bash
# Summary and category totals across multiple months
cargo run -p cli -- report range 2025-06..2025-08

# Expense only (default)
cargo run -p cli -- report range 2025-06..2025-08 --expense

# Income only
cargo run -p cli -- report range 2025-06..2025-08 --income

# Both (expense + income)
cargo run -p cli -- report range 2025-06..2025-08 --both
```

### Export to CSV
```bash
# Export all entries to CSV (UTF-8 with BOM, Excel-friendly)
cargo run -p cli -- export out.csv
```