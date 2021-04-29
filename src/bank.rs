use serde::Deserialize;
use std::collections::HashMap;
#[cfg(test)]
use rstest::*;

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum TransactionType {
    deposit,
    withdrawal,
    dispute,
    resolve,
    chargeback,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Transaction {
    pub kind: TransactionType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f32>,
    #[serde(default)]
    has_been_disputed: bool,
    #[serde(default)]
    has_been_resolved: bool,
}

impl Transaction {
    #[cfg(test)]
    pub fn new(kind: TransactionType, client: u16, tx: u32, amount: Option<f32>) -> Self {
        Self {
            kind,
            client,
            tx,
            amount,
            has_been_disputed: false,
            has_been_resolved: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Funds {
    available: f32,
    held: f32,
    is_locked: bool,
}

impl Funds {
    pub fn new(available: f32, held: f32) -> Self {
        Self {
            available,
            held,
            is_locked: false,
        }
    }
}

pub struct Bank {
    pub accounts: HashMap<u16, Funds>,
    pub ledger: HashMap<u32, Transaction>,
}

impl Bank {
    pub fn process_transaction(&mut self, transaction: &Transaction) -> Result<(), &str> {
        match transaction.kind {
            TransactionType::deposit => {
                self.credit_account(transaction.client, transaction.amount.unwrap())
            }
            TransactionType::withdrawal => {
                self.debit_account(transaction.client, transaction.amount.unwrap())
            }
            TransactionType::dispute => {
                self.dispute_transaction(transaction.client, transaction.tx)
            }
            TransactionType::resolve => {
                self.resolve_transaction(transaction.client, transaction.tx)
            }
            TransactionType::chargeback => {
                self.chargeback_transaction(transaction.client, transaction.tx)
            }
        }
    }

    pub fn add_transaction_to_ledger(&mut self, transaction: Transaction) {
        self.ledger.insert(transaction.tx, transaction);
    }

    pub fn print_accounts(&self) {
        println!("client, available, held, total, locked");
        for (client, funds) in self.accounts.iter() {
            println!(
                "{},{:.4},{:.4},{:.4},{}",
                client,
                funds.available,
                funds.held,
                funds.available + funds.held,
                funds.is_locked
            )
        }
    }

    fn credit_account(&mut self, client: u16, amount: f32) -> Result<(), &str> {
        let funds = self.accounts.entry(client).or_insert(Funds::new(0.0, 0.0));
        if funds.is_locked {
            return Err("Account frozen");
        };

        if funds.available + amount < f32::MAX {
            funds.available += amount;
            Ok(())
        } else {
            Err("Upper limit reached, time to give to charity?")
        }
    }

    fn debit_account(&mut self, client: u16, amount: f32) -> Result<(), &str> {
        let funds = self.accounts.entry(client).or_insert(Funds::new(0.0, 0.0));
        if funds.is_locked {
            return Err("Account frozen");
        };
        if funds.available > amount {
            funds.available -= amount;
            Ok(())

        } else {
            Err("Insufficient funds")
        }
    }

    fn dispute_transaction(&mut self, client: u16, tx: u32) -> Result<(), &str> {
        let disputed_transaction =
            check_for_valid_disputed_transaction(&mut self.ledger, client, tx)?;
        if disputed_transaction.has_been_disputed {
            return Err("Transaction is already disputed");
        };

        if let Some(funds) = self.accounts.get_mut(&client) {
            if funds.is_locked {
                return Err("Account frozen");
            };
            funds.held += disputed_transaction.amount.unwrap();
            disputed_transaction.has_been_disputed = true;
            Ok(())

        } else {
            return Err("Client not found");
        }
    }

    fn resolve_transaction(&mut self, client: u16, tx: u32) -> Result<(), &str> {
        let disputed_transaction =
            check_for_valid_disputed_transaction(&mut self.ledger, client, tx)?;
        if !disputed_transaction.has_been_disputed {
            return Err("Transaction is not disputed")
        };
        if disputed_transaction.has_been_resolved {
            return Err("Transaction already resolved")
        };

        if let Some(funds) = self.accounts.get_mut(&client) {
            if funds.is_locked {
                return Err("Account frozen");
            };
            funds.available += disputed_transaction.amount.unwrap();
            funds.held -= disputed_transaction.amount.unwrap();
            disputed_transaction.has_been_resolved = true;
            Ok(())

        } else {
            return Err("Client not found");
        }
    }

    fn chargeback_transaction(&mut self, client: u16, tx: u32) -> Result<(), &str> {
        let disputed_transaction =
            check_for_valid_disputed_transaction(&mut self.ledger, client, tx)?;
        if !disputed_transaction.has_been_disputed {
            return Err("Transaction is not disputed");
        };
        if disputed_transaction.has_been_resolved {
            return Err("Transaction already resolved")
        };

        if let Some(funds) = self.accounts.get_mut(&client) {
            funds.held -= disputed_transaction.amount.unwrap();
            funds.is_locked = true;
            disputed_transaction.has_been_disputed = false;
            Ok(())

        } else {
            return Err("Client not found");
        }
    }
}

fn check_for_valid_disputed_transaction(
    ledger: &mut HashMap<u32, Transaction>,
    client: u16,
    tx: u32,
) -> Result<&mut Transaction, &'static str> {
    if let Some(disputed_transaction) = ledger.get_mut(&tx) {
        if disputed_transaction.client != client {
            return Err("Dispute transaction of another client");
        } else if disputed_transaction.amount == None {
            return Err("Invalid transaction");
        } else {
            return Ok(disputed_transaction);
        };
    } else {
        return Err("Transaction not found");
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    pub struct Client {
        id: u16,
        funds: Funds,
    }

    #[fixture]
    pub fn bank() -> Bank {
        Bank {
            accounts: HashMap::new(),
            ledger: HashMap::new(),
        }
    }

    pub fn add_client(bank: &mut Bank) -> Client {
        let client = Client {
            id: 1,
            funds: Funds::new(40.0, 100.0),
        };
        bank.accounts.insert(client.id, client.funds.clone());
        client
    }

    pub fn add_locked_client(bank: &mut Bank) -> Client {
        let client = Client {
            id: 1,
            funds: Funds {
                available: 40.0,
                held: 100.0,
                is_locked: true,
            },
        };
        bank.accounts.insert(client.id, client.funds.clone());
        client
    }

    pub fn add_rich_client(bank: &mut Bank) -> Client {
        let client = Client {
            id: 1,
            funds: Funds::new(f32::MAX, 100.0),
        };
        bank.accounts.insert(client.id, client.funds.clone());
        client
    }

    pub fn add_valid_transaction(bank: &mut Bank) -> Transaction {
        let tx = 42_u32;
        let transaction = Transaction::new(TransactionType::deposit, 1, tx, Some(50.0));
        bank.ledger.insert(tx, transaction.clone());
        transaction
    }

    pub fn add_invalid_transaction(bank: &mut Bank) -> Transaction {
        let tx = 42_u32;
        let transaction = Transaction::new(TransactionType::dispute, 1, tx, None);
        bank.ledger.insert(tx, transaction.clone());
        transaction
    }

    pub fn add_disputed_transaction(bank: &mut Bank) -> Transaction {
        let tx = 42_u32;
        let transaction = Transaction {
            kind: TransactionType::deposit,
            client: 1,
            tx,
            amount: Some(50.0),
            has_been_disputed: true,
            has_been_resolved: false,
        };
        bank.ledger.insert(tx, transaction.clone());
        transaction
    }

    #[rstest]
    fn credit_non_existing_client_account(mut bank: Bank) {
        let client: u16 = 1;
        let amount = Some(42.1234);
        let transaction = Transaction::new(TransactionType::deposit, client, 1, amount);

        bank.process_transaction(&transaction).unwrap();

        assert_eq!(
            bank.accounts.get(&client).unwrap(),
            &Funds::new(amount.unwrap(), 0.0)
        );
    }

    #[rstest]
    fn credit_existing_client_account(mut bank: Bank) {
        let existing_client = add_client(&mut bank);
        let amount = Some(42.1234);
        let transaction = Transaction::new(TransactionType::deposit, existing_client.id, 1, amount);

        bank.process_transaction(&transaction).unwrap();

        assert_eq!(
            bank.accounts.get(&existing_client.id).unwrap(),
            &Funds::new(
                existing_client.funds.available + amount.unwrap(),
                existing_client.funds.held
            )
        );
    }

    #[rstest]
    fn credit_frozen_account(mut bank: Bank) {
        let locked_client = add_locked_client(&mut bank);
        let transaction = Transaction::new(TransactionType::deposit, locked_client.id, 1, Some(42.0));

        let result = bank.process_transaction(&transaction).unwrap_err();

        assert_eq!(result, "Account frozen");
    }

    #[rstest]
    fn credit_full_account(mut bank: Bank) {
        let rich_client = add_rich_client(&mut bank);
        let transaction = Transaction::new(TransactionType::deposit, rich_client.id, 1, Some(42.0));

        let result = bank.process_transaction(&transaction).unwrap_err();

        assert_eq!(result, "Upper limit reached, time to give to charity?");
    }

    #[rstest]
    fn debit_non_existing_client(mut bank: Bank) {
        let amount = Some(0.0);
        let transaction = Transaction::new(TransactionType::withdrawal, 1, 1, amount);

        let result = bank.process_transaction(&transaction).unwrap_err();

        assert_eq!(result, "Insufficient funds");
    }

    #[rstest]
    fn debit_client_account_with_sufficient_funds(mut bank: Bank) {
        let existing_client = add_client(&mut bank);
        let amount = Some(10.0);
        let withdrawal = Transaction::new(TransactionType::withdrawal, existing_client.id, 1, amount);
        bank.process_transaction(&withdrawal).unwrap();

        assert_eq!(
            bank.accounts.get(&existing_client.id).unwrap(),
            &Funds::new(
                existing_client.funds.available - amount.unwrap(),
                existing_client.funds.held
            )
        );
    }

    #[rstest]
    fn debit_client_account_with_insufficient_funds(mut bank: Bank) {
        let existing_client = add_client(&mut bank);
        let amount = Some(9999.9);
        let withdrawal = Transaction::new(TransactionType::withdrawal, existing_client.id, 1, amount);

        let result = bank.process_transaction(&withdrawal).unwrap_err();

        assert_eq!(result, "Insufficient funds");
        assert_eq!(
            bank.accounts.get(&existing_client.id).unwrap(),
            &Funds::new(existing_client.funds.available, existing_client.funds.held)
        );
    }

    #[rstest]
    fn dispute_existing_transaction(mut bank: Bank) {
        let existing_client = add_client(&mut bank);
        let existing_transaction = add_valid_transaction(&mut bank);

        let dispute = Transaction::new(
            TransactionType::dispute,
            existing_transaction.client,
            existing_transaction.tx,
            None,
        );

        bank.process_transaction(&dispute).unwrap();

        assert_eq!(
            bank.accounts.get(&existing_client.id).unwrap(),
            &Funds::new(
                existing_client.funds.available,
                existing_client.funds.held + existing_transaction.amount.unwrap()
            )
        );
        assert_eq!(
            bank.ledger
                .get(&existing_transaction.tx)
                .unwrap()
                .has_been_disputed,
            true
        );
    }

    #[rstest]
    fn dispute_existing_transaction_of_another_client(mut bank: Bank) {
        let existing_transaction = add_valid_transaction(&mut bank);

        let dispute = Transaction::new(
            TransactionType::dispute,
            existing_transaction.client + 1,
            existing_transaction.tx,
            None,
        );

        let result = bank.process_transaction(&dispute).unwrap_err();

        assert_eq!(result, "Dispute transaction of another client");
    }

    #[rstest]
    fn dispute_non_existing_transaction(mut bank: Bank) {
        let dispute = Transaction::new(TransactionType::dispute, 2, 123456789, None);

        let result = bank.process_transaction(&dispute).unwrap_err();

        assert_eq!(result, "Transaction not found");
    }

    #[rstest]
    fn dispute_already_disputed_transaction(mut bank: Bank) {
        add_client(&mut bank);
        let disputed_transaction = add_disputed_transaction(&mut bank);
        let resolve = Transaction::new(
            TransactionType::resolve,
            disputed_transaction.client,
            disputed_transaction.tx,
            None,
        );

        bank.process_transaction(&resolve).unwrap();
        let dispute = Transaction::new(
            TransactionType::dispute,
            disputed_transaction.client,
            disputed_transaction.tx,
            None,
        );

        let result = bank.process_transaction(&dispute).unwrap_err();

        assert_eq!(result, "Transaction is already disputed");
    }

    #[rstest]
    fn dispute_invalid_transaction(mut bank: Bank) {
        let existing_invalid_transaction = add_invalid_transaction(&mut bank);
        let dispute = Transaction::new(
            TransactionType::dispute,
            existing_invalid_transaction.client,
            existing_invalid_transaction.tx,
            None,
        );

        let result = bank.process_transaction(&dispute).unwrap_err();

        assert_eq!(result, "Invalid transaction");
    }

    #[rstest]
    fn resolve_disputed_transaction(mut bank: Bank) {
        let existing_client = add_client(&mut bank);
        let disputed_transaction = add_disputed_transaction(&mut bank);
        let resolve = Transaction::new(
            TransactionType::resolve,
            disputed_transaction.client,
            disputed_transaction.tx,
            None,
        );

        bank.process_transaction(&resolve).unwrap();

        assert_eq!(
            bank.accounts.get(&existing_client.id).unwrap(),
            &Funds::new(
                existing_client.funds.available + disputed_transaction.amount.unwrap(),
                existing_client.funds.held - disputed_transaction.amount.unwrap()
            )
        );
        assert_eq!(
            bank.ledger
                .get(&disputed_transaction.tx)
                .unwrap()
                .has_been_resolved,
            true
        );
    }

    #[rstest]
    fn resolve_already_resolved_transaction(mut bank: Bank) {
        add_client(&mut bank);
        let disputed_transaction = add_disputed_transaction(&mut bank);
        let resolve = Transaction::new(
            TransactionType::resolve,
            disputed_transaction.client,
            disputed_transaction.tx,
            None,
        );

        bank.process_transaction(&resolve).unwrap();
        let result = bank.process_transaction(&resolve).unwrap_err();

        assert_eq!(result, "Transaction already resolved");
    }


    #[rstest]
    fn resolve_non_disputed_transaction(mut bank: Bank) {
        let non_disputed_transaction = add_valid_transaction(&mut bank);
        let resolve = Transaction::new(
            TransactionType::resolve,
            non_disputed_transaction.client,
            non_disputed_transaction.tx,
            None,
        );

        let result = bank.process_transaction(&resolve).unwrap_err();

        assert_eq!(result, "Transaction is not disputed");
    }

    #[rstest]
    fn chargeback_disputed_transaction(mut bank: Bank) {
        let existing_client = add_client(&mut bank);
        let disputed_transaction = add_disputed_transaction(&mut bank);
        let resolve = Transaction::new(
            TransactionType::chargeback,
            disputed_transaction.client,
            disputed_transaction.tx,
            None,
        );

        bank.process_transaction(&resolve).unwrap();

        assert_eq!(
            bank.accounts.get(&existing_client.id).unwrap(),
            &Funds {
                available: existing_client.funds.available,
                held: existing_client.funds.held - disputed_transaction.amount.unwrap(),
                is_locked: true
            }
        );
    }
}
