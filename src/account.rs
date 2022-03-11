use std::collections::{HashMap};

use log::trace;
use rust_decimal::{Decimal};
use serde::Serialize;

use crate::errors::TransactionSystemError;
use crate::instructions::{Instruction, Transaction, Operation};
use crate::result::Result;

#[derive(Serialize, Debug, Default)]
pub struct Account {
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
    #[serde(skip)]
    txhistory: HashMap<u32, Transaction>,
}

impl Account {
    fn deposit(&mut self, data: Transaction) -> Result {
        trace!("client {} tx {} deposits {}", data.client(), data.tx(), data.amount());
        self.available += data.amount();
        self.total += data.amount();
        self.txhistory.insert(data.tx(), data);

        Ok(())
    }

    fn withdrawal(&mut self, mut data: Transaction) -> Result {
        trace!("client {} tx {} attempts withdraw {}", data.client(), data.tx(), data.amount());
        let mut available = self.available;
        available -= data.amount();
        if available >= Decimal::new(0, 0) {
            self.available = available;
            self.total -= data.amount();
            data.negate(); // That way we record transaction with the negative sign
            self.txhistory.insert(data.tx(), data);

            Ok(())
        } else {
            Err(TransactionSystemError::TransactionError{ 
                message: "attempt to withdraw more than available".to_owned(),
                transaction: data
            })
        }
    }

    fn dispute(&mut self, data: Operation) -> Result {
        trace!("client {} tx {} receives dispute", data.client(), data.tx());
        // Refer to `README.md` for information about disputes repeated for the same transaction
        if let Some(entry) = self.txhistory.get(&data.tx()) {
            entry.try_set_disputed().and_then( |_| {
                self.available -= entry.amount();
                self.held += entry.amount();
                Ok(())
            })
        } else {
            Err(TransactionSystemError::OperationError{
                message: "attempt to dispute non-existing transaction".to_owned(),
                operation: data
            })
        }
    }

    fn resolve(&mut self, data: Operation) -> Result {
        trace!("client {} tx {} resolves dispute", data.client(), data.tx());
        // Refer to `README.md` for information about resolves for transactions without disputes started
        if let Some(entry) = self.txhistory.get(&data.tx()) {
            entry.try_set_resolved().and_then(|_| {
                self.available += entry.amount();
                self.held -= entry.amount();
                Ok(())
            })

        } else {
            Err(TransactionSystemError::OperationError{
                message: "attempt to resolve non-existing transaction".to_owned(),
                operation: data
            })
        }
    }

    fn chargeback(&mut self, data: Operation) -> Result {
        trace!("client {} tx {} charges back of the dispute", data.client(), data.tx());
        // Refer to `README.md` for information about chargebacks for transactions without disputes started
        if let Some(entry) = self.txhistory.get(&data.tx()) {
            entry.try_set_chargedback().and_then(|_| {
                self.locked = true;
                self.total -= entry.amount();
                self.held -= entry.amount();
                Ok(())
            })
        } else {
            Err(TransactionSystemError::OperationError{
                message: "attempt to chargeback non-existing transaction".to_owned(),
                operation: data
            })
        }
    }

    pub fn apply(&mut self, instruction: Instruction) -> Result {
        match instruction {
            Instruction::Deposit(data)    => self.deposit(data),
            Instruction::Withdrawal(data) => self.withdrawal(data),
            Instruction::Dispute(data)    => self.dispute(data),
            Instruction::Resolve(data)    => self.resolve(data),
            Instruction::Chargeback(data) => self.chargeback(data),
        }
    }

    pub fn available(&self) -> Decimal {
        self.available
    }

    pub fn held(&self) -> Decimal {
        self.held
    }

    pub fn total(&self) -> Decimal {
        self.total
    }
    
    pub fn locked(&self) -> bool {
        self.locked
    }
}

#[cfg(test)]
mod test {
    use rust_decimal::{Decimal, prelude::FromPrimitive};
    use crate::instructions::{Transaction, Operation};
    use super::Account;

    #[test]
    fn deposit() {
        let mut account = Account::default();

        assert_eq!(account.available, Decimal::from_i32(0).unwrap());
        assert_eq!(account.held, Decimal::from_i32(0).unwrap());
        assert_eq!(account.total, Decimal::from_i32(0).unwrap());
        assert_eq!(account.locked, false);

        let data = Transaction::new(1, 1, Decimal::from_i32(30).unwrap() );
        assert!(account.deposit(data).is_ok());

        assert_eq!(account.available, Decimal::from_i32(30).unwrap());
        assert_eq!(account.held, Decimal::from_i32(0).unwrap());
        assert_eq!(account.total, Decimal::from_i32(30).unwrap());
    }

    #[test]
    fn withdrawal() {
        let mut account = Account::default();

        account.available = Decimal::new(1000, 1);
        assert_eq!(account.held, Decimal::from_i32(0).unwrap());
        account.total = Decimal::from_i32(120).unwrap();
        assert_eq!(account.locked, false);

        let data = Transaction::new(1, 1, Decimal::from_i32(30).unwrap() );
        assert!(account.withdrawal(data).is_ok());

        assert_eq!(account.available, Decimal::from_i32(70).unwrap());
        assert_eq!(account.held, Decimal::from_i32(0).unwrap());
        assert_eq!(account.total, Decimal::from_i32(90).unwrap());

        // Overdraft attempt
        let data = Transaction::new(1, 1, Decimal::from_i32(80).unwrap() );
        assert!(account.withdrawal(data).is_err());
    }

    #[test]
    fn dispute() {
        let mut account = Account::default();

        account.available = Decimal::new(1500, 1);
        assert_eq!(account.held, Decimal::from_i32(0).unwrap());
        account.total = Decimal::from_i32(150).unwrap();
        assert_eq!(account.locked, false);

        let data = Transaction::new(1, 1, Decimal::from_i32(50).unwrap() );
        assert!(account.deposit(data).is_ok());

        let data = Operation::new(1, 1);
        assert!(account.dispute(data).is_ok());

        assert_eq!(account.available, Decimal::from_i32(150).unwrap());
        assert_eq!(account.held, Decimal::from_i32(50).unwrap());
        assert_eq!(account.total, Decimal::from_i32(200).unwrap());
    }

    #[test]
    fn resolve() {
        let mut account = Account::default();

        account.available = Decimal::new(1500, 1);
        assert_eq!(account.held, Decimal::from_i32(0).unwrap());
        account.total = Decimal::from_i32(150).unwrap();
        assert_eq!(account.locked, false);

        let data = Transaction::new(1, 1, Decimal::from_i32(50).unwrap() );
        assert!(account.deposit(data).is_ok());

        let data = Operation::new(1, 1);
        assert!(account.dispute(data).is_ok());

        let data = Operation::new(1, 1);
        assert!(account.resolve(data).is_ok());

        assert_eq!(account.available, Decimal::from_i32(200).unwrap());
        assert_eq!(account.held, Decimal::from_i32(0).unwrap());
        assert_eq!(account.total, Decimal::from_i32(200).unwrap());
    }

    #[test]
    fn chargeback() {
        let mut account = Account::default();

        account.available = Decimal::new(1500, 1);
        assert_eq!(account.held, Decimal::from_i32(0).unwrap());
        account.total = Decimal::from_i32(150).unwrap();
        assert_eq!(account.locked, false);

        let data = Transaction::new(1, 1, Decimal::from_i32(50).unwrap() );
        assert!(account.deposit(data).is_ok());

        let data = Operation::new(1, 1);
        assert!(account.dispute(data).is_ok());

        let data = Operation::new(1, 1);
        assert!(account.chargeback(data).is_ok());

        assert_eq!(account.available, Decimal::from_i32(150).unwrap());
        assert_eq!(account.held, Decimal::from_i32(0).unwrap());
        assert_eq!(account.total, Decimal::from_i32(150).unwrap());
        assert!(account.locked);
    }
}