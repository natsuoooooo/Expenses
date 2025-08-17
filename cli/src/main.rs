use std::fs::File;
use std::io::{BufWriter, Write};
use csv::WriterBuilder;
use ledger_module::{
    add_entry, category_totals_by_kind, category_totals_by_kind_in_range,
    delete_entry, init_db, list_entries, month_summary, open_db,
    summary_in_range, Kind, entries_in_month, entries_in_range, Entry
};

fn current_ym() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m").to_string()
}

fn parse_ym_range(s: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = s.split("..").collect();
    if parts.len() != 2 {
        return None;
    }

    let a = parts[0];
    let b = parts[1];
    if a.len() == 7 && &a[4..5] == "-" && b.len() == 7 && &b[4..5] == "-" {
        Some((a.to_string(), b.to_string()))
    } else {
        None
    }
}

fn write_csv(path: &str, rows: &[Entry]) -> csv::Result<()> {
    let file = File::create(path)?;
    let mut buf = BufWriter::new(file);

    buf.write_all(b"\xEF\xBB\xBF")?;

    let mut wtr = WriterBuilder::new().from_writer(buf);

    wtr.write_record(&["id", "kind", "amount", "category", "note", "created_at"])?;

    for e in rows {
        let kind = if e.kind == Kind::Expense { "expense" } else { "income" };
        wtr.write_record(&[
            e.id.to_string(),
            kind.to_string(),
            e.amount.to_string(),
            e.category.to_string(),
            e.note.clone().unwrap_or_default(),
            e.created_at.to_string(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
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
            match delete_entry(&conn, id) {
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
                "range" => {
                    if args.len() < 4 {
                        eprintln!("Usage: {} report range <YYYY-MM..YYYY-MM> [--income|--expense|--both]", args[0]);
                        return;
                    }
                    let range = &args[3];
                    let Some((start_ym, end_ym)) = parse_ym_range(range) else {
                        eprintln!("Invalid range: {} (expected YYYY-MM..YYYY-MM)", range);
                        return;
                    };

                    let flag = args.iter().find(|a| a.starts_with("--")).map(|s| s.as_str());

                    match summary_in_range(&conn, &start_ym, &end_ym) {
                        Ok(s) => {
                            println!("== Summary {}..{} ==", s.start_month, s.end_month);
                            println!("Income : {}", s.income);
                            println!("Expense: {}", s.expense);
                            println!("Balance: {}", s.balance);
                        }
                        Err(e) => {
                            eprintln!("Failed to get range summary: {}", e);
                            return;
                        }
                    }

                    match flag {
                        Some("--both") => {
                            let exp = category_totals_by_kind_in_range(&conn, &start_ym, &end_ym, Kind::Expense)
                                .unwrap_or_default();
                            let inc = category_totals_by_kind_in_range(&conn, &start_ym, &end_ym, Kind::Income)
                                .unwrap_or_default();

                            println!();
                            println!("== Category Totals (Expense) {}..{} ==", start_ym, end_ym);
                            if exp.is_empty() { println!("(no data)"); }
                            for r in exp { println!("{:12} {}", r.category, r.total); }

                            println!();
                            println!("== Category Totals (Income)  {}..{} ==", start_ym, end_ym);
                            if inc.is_empty() { println!("(no data)"); }
                            for r in inc { println!("{:12} {}", r.category, r.total); }
                        }
                        Some("--income") => {
                            let rows = category_totals_by_kind_in_range(&conn, &start_ym, &end_ym, Kind::Income)
                                .unwrap_or_default();
                            println!();
                            println!("== Category Totals (Income) {}..{} ==", start_ym, end_ym);
                            if rows.is_empty() { println!("(no data)"); }
                            for r in rows { println!("{:12} {}", r.category, r.total); }
                        }
                        _ => {
                            let rows = category_totals_by_kind_in_range(&conn, &start_ym, &end_ym, Kind::Expense)
                                .unwrap_or_default();
                            println!();
                            println!("== Category Totals (Expense) {}..{} ==", start_ym, end_ym);
                            if rows.is_empty() { println!("(no data)"); }
                            for r in rows { println!("{:12} {}", r.category, r.total); }
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
        "export" => {
            if args.len() < 3 {
                eprintln!("Usage: {} export <csv> <month|range> [YYYY-MM|YYYY-MM..YYYY-MM]", args[0]);
                return;
            }
            match args[2].as_str() {
                "csv" => {
                    if args.len() < 4 {
                        eprintln!("Usage: {} export csv <month [YYYY-MM]|range YYYY-MM..YYYY-MM>", args[0]);
                        return;
                    }
                    match args[3].as_str() {
                        "month" => {
                            // 5番目に YYYY-MM があれば使い、無ければ当月
                            let ym = if args.len() >= 5 { args[4].clone() } else { current_ym() };
                            let rows = entries_in_month(&conn, &ym).expect("fetch month entries");
                            let filename = format!("export_month_{}.csv", ym);
                            write_csv(&filename, &rows).expect("write csv");
                            println!("Exported to {}", filename);
                        }
                        "range" => {
                            if args.len() < 5 {
                                eprintln!("Usage: {} export csv range <YYYY-MM..YYYY-MM>", args[0]);
                                return;
                            }
                            let range = &args[4];
                            let Some((start_ym, end_ym)) = parse_ym_range(range) else {
                                eprintln!("Invalid range: {} (expected YYYY-MM..YYYY-MM)", range);
                                return;
                            };
                            let rows = entries_in_range(&conn, &start_ym, &end_ym).expect("fetch range entries");
                            let filename = format!("export_range_{}..{}.csv", start_ym, end_ym);
                            write_csv(&filename, &rows).expect("write csv");
                                println!("Exported to {}", filename);
                        }
                        _ => {
                            eprintln!("Usage: {} export csv <month|range> ...", args[0]);
                        }
                    }
                }
                _ => eprintln!("Usage: {} export <csv> ...", args[0]),
            }
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
        }
    }
}
