use serde::Deserialize;

use std::collections::HashMap;
use std::fmt;

pub struct ExpenseReport {
    participants: Vec<String>,
    participant_indices: HashMap<String, usize>,
    exchange_rates: HashMap<String, f64>,
    base_currency: String,
    transactions: Vec<Transaction>,
    history: Vec<String>,
}

impl ExpenseReport {
    pub fn new(json: &str) -> Self {
        let input: JsonInput = serde_json::from_str(json).unwrap();

        let participant_indices = input
            .participants
            .iter()
            .enumerate()
            .map(|(id, name)| (name.clone(), id))
            .collect();

        let mut exchange_rates: HashMap<String, f64> = HashMap::new();

        exchange_rates.insert(input.currency.clone(), 1.00);

        for (currency, relation_to_base) in input.exchange_rates {
            exchange_rates.insert(currency, relation_to_base);
        }

        let mut report = Self {
            participants: input.participants,
            participant_indices,
            exchange_rates,
            base_currency: input.currency,
            transactions: Vec::new(),
            history: Vec::new(),
        };

        for transfer in input.transfers.into_iter() {
            report.transfer(
                &transfer.from,
                &transfer.to,
                &transfer.what,
                transfer.amount,
                &transfer.currency,
            );
        }

        for expense in input.expenses.into_iter() {
            report.expense(
                &expense.by,
                &expense.split,
                &expense.what,
                expense.amount,
                &expense.currency,
            );
        }

        report
    }

    pub fn summarize(&self) -> &[String] {
        &self.history
    }

    pub fn balance(&self) -> (Vec<(String, String, f64)>, Vec<(String, f64)>) {
        let mut balances: Vec<i64> = vec![0; self.participants.len()];

        for transaction in self.transactions.iter() {
            balances[transaction.participant_index] += transaction.amount_minor;
        }

        let mut to_be_done = Vec::new();

        for i in 0..balances.len() - 1 {
            let mut j = i + 1;

            while balances[i] != 0 && j < balances.len() {
                let max_balacing_amount = ((balances[i] * balances[j]).signum() < 0)
                    .then_some(balances[i].signum() * balances[i].abs().min(balances[j].abs()));

                if let Some(amount) = max_balacing_amount {
                    balances[i] -= amount;
                    balances[j] += amount;

                    let (from, to) = if amount > 0 {
                        (self.participants[i].clone(), self.participants[j].clone())
                    } else {
                        (self.participants[j].clone(), self.participants[i].clone())
                    };

                    to_be_done.push((from, to, (amount.abs() as f64) / 100.0));
                }

                j += 1;
            }
        }

        let residuals = balances
            .into_iter()
            .enumerate()
            .filter_map(|(id, amount_minor)| {
                (amount_minor != 0)
                    .then_some((self.participants[id].clone(), (amount_minor as f64) / 100.0))
            })
            .collect();

        (to_be_done, residuals)
    }

    pub fn base_currency(&self) -> &str {
        &self.base_currency
    }

    fn transfer(&mut self, from: &str, to: &str, what: &str, amount: f64, currency: &str) {
        let rate = self.get_exchange_rate(currency);

        let from = self.get_participant_index(from);
        let to = self.get_participant_index(to);

        self.history.push(format!(
            "{} gave {:.02} {}{} to {} for '{}'.",
            self.participants[from],
            amount,
            currency,
            self.base_currency_text(amount, currency),
            self.participants[to],
            what
        ));

        let trans_out = Transaction::new(from, -amount * rate);
        let trans_in = Transaction::new(to, amount * rate);

        self.transactions.push(trans_out);
        self.transactions.push(trans_in);
    }

    fn expense(
        &mut self,
        by: &str,
        participant_split: &HashMap<String, u32>,
        what: &str,
        amount: f64,
        currency: &str,
    ) {
        let mut split = vec![0; self.participants.len()];

        for (participant, share_num) in participant_split {
            split[self.get_participant_index(participant)] = *share_num;
        }

        let split_den: u32 = split.iter().sum();

        assert_ne!(split_den, 0, "Expense '{}' is not shared by anyone", what);

        let rate = self.get_exchange_rate(currency);

        let by = self.get_participant_index(by);

        let trans_out = Transaction::new(by, -amount * rate);

        let mut entry = Vec::new();

        entry.push(format!(
            "{} paid {:.02} {}{} for '{}', which is split:",
            self.participants[by],
            amount,
            currency,
            self.base_currency_text(amount, currency),
            what
        ));

        self.transactions.push(trans_out);

        for (i, split_num) in split.iter().enumerate().filter(|(_, n)| **n != 0) {
            let share = *split_num as f64 / split_den as f64;

            let trans_in = Transaction::new(i, amount * share * rate);

            entry.push(format!(
                "    {} {}/{} ({} {})",
                self.participants[i], split_num, split_den, trans_in, self.base_currency
            ));

            self.transactions.push(trans_in);
        }

        self.history.push(entry.join("\n"));
    }

    fn get_participant_index(&self, participant: &str) -> usize {
        self.participant_indices
            .get(participant)
            .expect(&format!(
                "Participant '{}' is in participant list",
                participant
            ))
            .clone()
    }

    fn get_exchange_rate(&self, currency: &str) -> f64 {
        self.exchange_rates
            .get(currency)
            .expect(&format!(
                "Currency '{}' not defined in exchange rates",
                currency
            ))
            .clone()
    }

    fn base_currency_text(&self, amount: f64, currency: &str) -> String {
        if currency != self.base_currency {
            let rate = self.get_exchange_rate(currency);
            let trans_dummy = Transaction::new(usize::MAX, amount.abs() * rate);

            format!(" ({} {})", trans_dummy, self.base_currency)
        } else {
            String::new()
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Transaction {
    participant_index: usize,
    amount_minor: i64,
}

impl Transaction {
    fn new(participant_index: usize, amount: f64) -> Self {
        Self {
            participant_index,
            amount_minor: (amount * 100.0).round() as i64,
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{:02}",
            self.amount_minor / 100,
            self.amount_minor % 100
        )
    }
}

#[derive(Deserialize, Debug)]
struct JsonInput {
    currency: String,
    exchange_rates: HashMap<String, f64>,
    participants: Vec<String>,
    transfers: Vec<JsonTransfer>,
    expenses: Vec<JsonExpense>,
}

#[derive(Deserialize, Debug)]
struct JsonTransfer {
    from: String,
    to: String,
    amount: f64,
    currency: String,
    what: String,
}

#[derive(Deserialize, Debug)]
struct JsonExpense {
    by: String,
    amount: f64,
    currency: String,
    what: String,
    split: HashMap<String, u32>,
}