use ledger_module::{open_db, init_db, add_entry, list_entries, Kind};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let conn = open_db().expect("Failed to open database");
    init_db(&conn).expect("Failed to initialize database");

    if args.len() < 2 {
        eprintln!("Usage: {} <command> [add|list...]", args[0]);
        return;
    }

    match args[1].as_str() {
        "add" => {
            if args.len() < 5 {
                eprintln!("Usage: {} add <kind> <amount> <category> [note]", args[0]);
                return;
            }
            let kind = match args[2].as_str() {
                "expense" => Kind::Expense,
                "income" => Kind::Income,
                _ => {
                    eprintln!("Invalid kind: {}. Use 'expense' or 'income'.", args[2]);
                    return;
                }
            };
            let amount: i64 = match args[3].parse() {
                Ok(num) if num > 0 => num,
                _ => {
                    eprintln!("Invalid amount: {}", args[3]);
                    return;
                }
            };
            let category = &args[4];
            let note = if args.len() > 5 {
                Some(args[5..].join(" "))
            } else {
                None
            };

            add_entry(&conn, kind, amount, category, note.as_deref()).expect("Failed to add entry");
            println!("Entry added successfully.");
        },
        "list" => {
            match list_entries(&conn) {
                Ok(entries) => {
                    for entry in entries {
                        println!("{}: {} {} {} {} [{}]", entry.created_at, if entry.kind == Kind::Expense { "Expense" } else { "Income" }, entry.amount, entry.category, entry.note.as_deref().unwrap_or(""), entry.id);
                    }
                },
                Err(e) => eprintln!("Failed to list entries: {}", e),
            }
        },
        "delete" => {
            if args.len() < 3 {
                eprintln!("Usage: {} delete <id>", args[0]);
                return;
            }
            let id: i64 = match args[2].parse() {
                Ok(num) => num,
                Err(_) => {
                    eprintln!("Invalid ID: {}", args[2]);
                    return;
                }
            };
            match ledger_module::delete_entry(&conn, id) {
                Ok(rows) if rows > 0 => println!("Entry deleted successfully."),
                Ok(_) => println!("No entry found with ID: {}", id),
                Err(e) => eprintln!("Failed to delete entry: {}", e),
            }
        },
        _ => {
            eprintln!("Unknown command: {}. Use 'add' or 'list'.", args[1]);
        } 
    }
}
