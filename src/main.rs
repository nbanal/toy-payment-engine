use std::{collections::HashMap, io};
use std::env;
use std::fs::File;
use std::io::BufReader;
mod bank;

fn main() -> io::Result<()> {
    let mut bank = bank::Bank {
        accounts: HashMap::new(),
        ledger: HashMap::new(),
    };

    let csv_filename = env::args().nth(1);
    let file = File::open(csv_filename.unwrap())?;
    let buffer_size = 1000;
    let reader = BufReader::with_capacity(buffer_size,file);

    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader);

        for record in rdr.deserialize() {
            match record as Result<bank::Transaction, csv::Error> {
                Ok(transaction) => {
                    if (transaction.kind == bank::TransactionType::deposit
                        || transaction.kind == bank::TransactionType::withdrawal)
                        && transaction.amount.is_none()
                    {
                        println!("Amount is missing");
                        continue;
                    }

                    match bank.process_transaction(&transaction) {
                        Ok(_) => {
                            if transaction.kind == bank::TransactionType::deposit
                                || transaction.kind == bank::TransactionType::withdrawal
                            {
                                bank.add_transaction_to_ledger(transaction);
                            }
                        }
                        Err(e) => println!("{}", e),
                    }
                }
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
            }
        }

    bank.print_accounts();
    Ok(())
}
