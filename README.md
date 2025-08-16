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

---

## CLI Quick Start

### Add entry
```bash
# Expense
cargo run -p cli -- add expense 1200 food Lunch

# Income
cargo run -p cli -- add income 50000 salary "August salary"
