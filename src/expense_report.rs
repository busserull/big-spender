use std::fmt;

pub struct ExpenseReport {
    participants: Vec<String>,
    exchange_rates: Vec<(String, f64)>,
    base_currency: String,
    transactions: Vec<Transaction>,
    history: Vec<String>,
}

impl ExpenseReport {
    pub fn new(participants: &[&str], base_currency: &str, rates: &[(&str, f64)]) -> Self {
        let participants = participants
            .into_iter()
            .map(|name| name.to_string())
            .collect();

        let mut exchange_rates = vec![(String::from(base_currency), 1.00)];

        for (currency, relation_to_base) in rates {
            exchange_rates.push((currency.to_string(), *relation_to_base));
        }

        Self {
            participants,
            exchange_rates,
            base_currency: String::from(base_currency),
            transactions: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn transfer(&mut self, from: &str, to: &str, what: &str, amount: f64, currency: &str) {
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

    pub fn expense(
        &mut self,
        by: &str,
        participant_split: &[(&str, u32)],
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

    pub fn summarize(&self) -> &[String] {
        &self.history
    }

    pub fn balance(&self) -> Vec<(String, String, f64)> {
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

        to_be_done
    }

    fn get_participant_index(&self, participant: &str) -> usize {
        self.participants
            .iter()
            .position(|name| name == participant)
            .expect(&format!("Participant '{}' is unknown", participant))
    }

    fn get_exchange_rate(&self, currency: &str) -> f64 {
        self.exchange_rates
            .iter()
            .find(|(face, _)| face == currency)
            .map(|(_, scale)| *scale)
            .expect(&format!(
                "Currency '{}' not defined in exchange rates",
                currency
            ))
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
