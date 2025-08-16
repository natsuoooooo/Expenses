A simple expense tracker in Rust.

- `cli/` : CLI app using SQLite
- `ledger_module/` : Core module (DB/domain logic)

## CLI quick start
```bash
cargo run -p cli -- add expense 1200 food Lunch
cargo run -p cli -- list
```