use dev_scrub::{CleanOptions, DevScanner, TerminalHud, VacuumEngine};
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let mut path = PathBuf::from(".");
    let mut command = "scan".to_string();
    let mut older_than: Option<u64> = None;
    let mut force_delete = false;
    let mut json_output = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "scan" => {
                command = "scan".to_string();
            }
            "clean" => {
                command = "clean".to_string();
            }
            "--force" => {
                force_delete = true;
            }
            "--json" => {
                json_output = true;
            }
            "--older-than" => {
                if i + 1 < args.len() {
                    if let Ok(days) = args[i + 1].parse::<u64>() {
                        older_than = Some(days);
                        i += 1;
                    }
                }
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            p if !p.starts_with('-') => {
                path = PathBuf::from(p);
            }
            _ => {}
        }
        i += 1;
    }

    let scanner = DevScanner::with_defaults();

    if !json_output {
        println!(
            "⚡ dev-scrub: Scanning '{}' for reclaimable developer artifacts...",
            path.display()
        );
    }

    let report = scanner.scan_path(&path);

    if command == "scan" {
        if json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_default()
            );
        } else {
            print!("{}", TerminalHud::format_scan_report(&report));
        }
    } else if command == "clean" {
        let clean_opts = CleanOptions {
            dry_run: !force_delete,
            min_days_inactive: older_than,
            filter_types: None,
        };

        let result = VacuumEngine::execute(&report.items, &clean_opts);

        if json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&result).unwrap_or_default()
            );
        } else {
            print!("{}", TerminalHud::format_clean_result(&result));
        }
    }
}

fn print_help() {
    println!("dev-scrub — Surgical Developer Disk Hygiene, Artifact Cleaner & Git Cache Vacuum");
    println!("\nUSAGE:");
    println!("    dev-scrub [scan|clean] [PATH] [OPTIONS]");
    println!("\nCOMMANDS:");
    println!("    scan [PATH]          Scan directory and display artifact breakdown (default)");
    println!("    clean [PATH]         Vacuum build artifacts (defaults to dry-run)");
    println!("\nOPTIONS:");
    println!("    --older-than <DAYS>  Only scrub projects dormant for at least <DAYS>");
    println!("    --force              Execute actual deletion (without --force, runs dry-run)");
    println!("    --json               Output machine-readable JSON telemetry");
    println!("    -h, --help           Display this help message");
    println!("\nEXAMPLES:");
    println!("    dev-scrub scan /root/work");
    println!("    dev-scrub clean /root/work --older-than 14");
    println!("    dev-scrub clean /root/work --force");
}
