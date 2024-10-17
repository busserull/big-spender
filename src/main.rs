/*
#[derive(Clone, Copy, Debug)]
enum Currency {
    NOK,
}

#[derive(Clone, Copy, Debug)]
struct Worth {
    currency: Currency,
    major: i32,
    minor: i32,
}
*/

type Worth = f64;

#[derive(Clone, Copy, Debug)]
enum ValueKind {
    Monetary,
    Derived,
}

#[derive(Clone, Debug)]
struct Value {
    frozen: bool,
    kind: ValueKind,
    what: String,
    worth: Worth,
}

impl Value {
    fn monetary(what: &str, worth: Worth) -> Self {
        Self {
            frozen: false,
            kind: ValueKind::Monetary,
            what: String::from(what),
            worth,
        }
    }

    fn derived(what: &str, worth: Worth) -> Self {
        Self {
            frozen: false,
            kind: ValueKind::Derived,
            what: String::from(what),
            worth,
        }
    }

    fn freeze(&mut self) {
        self.frozen = true;
    }
}

impl<'a> std::iter::Sum<&'a Value> for Worth {
    fn sum<I: Iterator<Item = &'a Value>>(iter: I) -> Self {
        iter.into_iter().fold(0.0, |acc, v| acc + v.worth)
    }
}

struct Group(Vec<(String, Vec<Value>)>);

impl Group {
    fn new(participants: &[&str]) -> Self {
        Self(
            participants
                .into_iter()
                .map(|name| (name.to_string(), Vec::new()))
                .collect(),
        )
    }

    fn transfer(&mut self, from: usize, to: usize, what: &str, worth: Worth) {
        self.0[from].1.push(Value::monetary(what, -worth));
        self.0[to].1.push(Value::monetary(what, worth));
    }

    fn expense(&mut self, by: usize, share: &[u32], what: &str, worth: Worth) {
        assert_eq!(
            share.len(),
            self.0.len(),
            "Share not specified for all participants"
        );

        self.0[by].1.push(Value::monetary(what, -worth));

        let share_den: u32 = share.iter().sum();

        for (i, share_num) in share.iter().enumerate().filter(|(_, n)| **n != 0) {
            let fraction = *share_num as Worth / share_den as Worth;

            self.0[i].1.push(Value::derived(what, worth * fraction));
        }
    }

    fn nameof(&self, participant_index: usize) -> String {
        self.0[participant_index].0.clone()
    }

    fn balance(&self) -> Vec<(String, String, Worth)> {
        let mut balances: Vec<Worth> = self
            .0
            .iter()
            .map(|(_, transactions)| transactions.iter().sum())
            .collect();

        let mut balance_transactions = Vec::new();

        for i in 0..balances.len() - 1 {
            let mut j = i;

            while balances[i].abs() > 0.01 {
                j += 1;

                if let Some(transfer) = max_balancing_amount(balances[i], balances[j]) {
                    balances[i] -= transfer;
                    balances[j] += transfer;

                    let (from, to, amount) = if transfer > 0.0 {
                        (self.nameof(i), self.nameof(j), transfer)
                    } else {
                        (self.nameof(j), self.nameof(i), -transfer)
                    };

                    balance_transactions.push((from, to, amount));
                }
            }
        }

        balance_transactions
    }
}

fn max_balancing_amount(a: Worth, b: Worth) -> Option<Worth> {
    ((a * b).signum() < 0.0).then_some(a.signum() * a.abs().min(b.abs()))
}

fn main() {
    let mut group = Group::new(&["Alice", "Bob", "Charlie", "Deidre", "DG"]);

    let alice = 0;
    let bob = 1;
    let charlie = 2;
    let deidre = 3;
    let dg = 4;

    /* Deposits */

    group.transfer(alice, dg, "Deposit", 20.0);
    group.transfer(bob, dg, "Deposit", 20.0);
    group.transfer(deidre, dg, "Deposit", 8.0);

    /* DG pays for stay */

    group.expense(dg, &[1, 1, 1, 1, 0], "Stay", 52.0);

    /* Charlie pays for a lot of food, but alice doesn't eat any of it */

    group.expense(charlie, &[0, 1, 1, 1, 0], "Food", 120.0);

    /* Do balancing */

    for (from, to, amount) in group.balance() {
        println!("{} pays {} {} kr", from, to, amount);
    }
}
