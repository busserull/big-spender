mod expense_report;

use clap::Parser;
use expense_report::ExpenseReport;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Print a summary of the cost report
    #[arg(short, long, default_value_t = true)]
    summarize: bool,

    /// Print a set of transfers between participants to balance the expenses
    #[arg(short, long, default_value_t = true)]
    balance: bool,

    /// Report the final residual (unbalanced) amount
    #[arg(short, long, default_value_t = false)]
    residual: bool,

    /// Cost report json file
    cost_report: PathBuf,
}

fn main() {
    let args = Cli::parse();

    let json = fs::read_to_string(args.cost_report).expect("Cannot read cost report file");

    let expense_report = ExpenseReport::new(&json);

    if args.summarize {
        println!("--- Cost summary ---");

        for entry in expense_report.summarize() {
            println!("\n{}", entry);
        }
    }

    let (balance_transactions, residuals) = expense_report.balance();

    if args.balance {
        if args.summarize {
            println!();
        }

        println!("--- To balance the expenses, do this ---");

        for (from, to, amount) in balance_transactions {
            println!(
                "\n{} pays {} {:.02} {}",
                from,
                to,
                amount,
                expense_report.base_currency()
            );
        }
    }

    if args.residual {
        if args.summarize || args.balance {
            println!();
        }

        println!("--- Residual amounts ---");

        if residuals.is_empty() {
            println!("[No residual amounts]");
        }

        for (participant, amount) in residuals {
            println!(
                "\n{}: {:.02} {}",
                participant,
                amount,
                expense_report.base_currency()
            );
        }
    }
}
