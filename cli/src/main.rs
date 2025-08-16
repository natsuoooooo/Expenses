use ledger_module::{
    Kind, add_entry, category_totals_by_kind, init_db, list_entries, month_summary, open_db,
};

fn current_ym() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m").to_string()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let conn = open_db().expect("Failed to open database");
    init_db(&conn).expect("Failed to initialize database");

    if args.len() < 2 {
        eprintln!("Usage: {} <command> [add|list|delete|report ...]", args[0]);
        return;
    }

    match args[1].as_str() {
        "add" => {
            if args.len() < 5 {
                eprintln!(
                    "Usage: {} add <expense|income> <amount> <category> [note...]",
                    args[0]
                );
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
        }

        "list" => match list_entries(&conn) {
            Ok(entries) => {
                for entry in entries {
                    let k = if entry.kind == Kind::Expense {
                        "Expense"
                    } else {
                        "Income"
                    };
                    println!(
                        "{}: {} {} {} {} [{}]",
                        entry.created_at,
                        k,
                        entry.amount,
                        entry.category,
                        entry.note.as_deref().unwrap_or(""),
                        entry.id
                    );
                }
            }
            Err(e) => eprintln!("Failed to list entries: {}", e),
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
        }

        "report" => {
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} report <month|category> [YYYY-MM] [--income|--expense|--both]",
                    args[0]
                );
                return;
            }
            let ym = if args.len() >= 4 && !args[3].starts_with("--") {
                args[3].clone()
            } else {
                current_ym()
            };

            match args[2].as_str() {
                "month" => match month_summary(&conn, &ym) {
                    Ok(s) => {
                        println!("== Summary {} ==", s.month);
                        println!("Income : {}", s.income);
                        println!("Expense: {}", s.expense);
                        println!("Balance: {}", s.balance);
                    }
                    Err(e) => eprintln!("Failed to get month summary: {}", e),
                },

                "category" => {
                    let flag = args
                        .iter()
                        .find(|a| a.starts_with("--"))
                        .map(|s| s.as_str());
                    match flag {
                        Some("--both") => {
                            let exp = category_totals_by_kind(&conn, &ym, Kind::Expense)
                                .unwrap_or_default();
                            let inc = category_totals_by_kind(&conn, &ym, Kind::Income)
                                .unwrap_or_default();

                            println!("== Category Totals (Expense) {} ==", ym);
                            if exp.is_empty() {
                                println!("(no data)");
                            }
                            for r in exp {
                                println!("{:12} {}", r.category, r.total);
                            }

                            println!();
                            println!("== Category Totals (Income)  {} ==", ym);
                            if inc.is_empty() {
                                println!("(no data)");
                            }
                            for r in inc {
                                println!("{:12} {}", r.category, r.total);
                            }
                        }
                        Some("--income") => {
                            let rows = category_totals_by_kind(&conn, &ym, Kind::Income)
                                .unwrap_or_default();
                            println!("== Category Totals (Income) {} ==", ym);
                            if rows.is_empty() {
                                println!("(no data)");
                            }
                            for r in rows {
                                println!("{:12} {}", r.category, r.total);
                            }
                        }
                        _ => {
                            let rows = category_totals_by_kind(&conn, &ym, Kind::Expense)
                                .unwrap_or_default();
                            println!("== Category Totals (Expense) {} ==", ym);
                            if rows.is_empty() {
                                println!("(no data)");
                            }
                            for r in rows {
                                println!("{:12} {}", r.category, r.total);
                            }
                        }
                    }
                }

                _ => {
                    eprintln!(
                        "Unknown report type: {}. Use 'month' or 'category'.",
                        args[2]
                    );
                }
            }
        }

        _ => {
            eprintln!("Unknown command: {}. Use 'add' or 'list'.", args[1]);
        }
    }
}
