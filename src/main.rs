mod expense_report;

use expense_report::ExpenseReport;

fn main() {
    let mut trip = ExpenseReport::new(
        &["Alice", "Bob", "Charlie", "Deidre", "DG"],
        "nok",
        &[("eur", 11.8375)],
    );

    /* Deposits */

    trip.transfer("Alice", "DG", "deposit", 20.0, "nok");
    trip.transfer("Bob", "DG", "deposit", 20.0, "nok");
    trip.transfer("Deidre", "DG", "deposit", 8.0, "eur");

    /* DG pays for stay */

    trip.expense(
        "DG",
        &[("Alice", 1), ("Bob", 1), ("Charlie", 1), ("Deidre", 1)],
        "stay",
        52.0,
        "nok",
    );

    /* Charlie pays for a lot of food, but alice doesn't eat any of it */

    trip.expense(
        "Charlie",
        &[("Bob", 1), ("Charlie", 1), ("Deidre", 1)],
        "food",
        120.0,
        "nok",
    );

    /* Do balancing */

    for entry in trip.summarize() {
        println!();
        println!("{}", entry);
    }

    println!();

    for (from, to, amount) in trip.balance() {
        println!("{} pays {} {:.02} nok", from, to, amount);
    }
}
