use std::collections::{HashMap};

use rust_decimal::{Decimal};
use serde::Serialize;

use crate::input::{Instruction, Transaction, Operation};

#[derive(Serialize, Debug, Default)]
pub struct Account {
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
    txhistory: HashMap<u32, Transaction>,
}

impl Account {
    fn deposit(&mut self, data: Transaction) {
        self.available += data.amount();
        self.total += data.amount();
        self.txhistory.insert(data.tx(), data);
    }

    fn withdrawal(&mut self, mut data: Transaction) {
        let mut available = self.available;
        available -= data.amount();
        if available >= Decimal::new(0, 0) {
            self.available = available;
            self.total -= data.amount();
            data.negate(); // That way we record transaction with the negative sign
            self.txhistory.insert(data.tx(), data);
        }
        // Ignoring potential error here
    }

    fn dispute(&mut self, data: Operation) {
        if let Some(entry) = self.txhistory.get(&data.tx()) {
            self.available -= entry.amount();
            self.held += entry.amount();
        }
    }

    fn resolve(&mut self, data: Operation) {
        if let Some(entry) = self.txhistory.get(&data.tx()) {
            self.available += entry.amount();
            self.held -= entry.amount();
        }
    }

    fn chargeback(&mut self, data: Operation) {
        if let Some(entry) = self.txhistory.get(&data.tx()) {
            self.locked = true;
            self.total -= entry.amount();
            self.held -= entry.amount();
        }
    }

    pub fn apply(&mut self, instruction: Instruction) {
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
    use crate::input::{Transaction, Operation};
    use super::Account;

    #[test]
    fn deposit() {
        let mut account = Account::default();

        assert_eq!(account.available, Decimal::from_i32(0).unwrap());
        assert_eq!(account.held, Decimal::from_i32(0).unwrap());
        assert_eq!(account.total, Decimal::from_i32(0).unwrap());
        assert_eq!(account.locked, false);

        let data = Transaction::new(1, 1, Decimal::from_i32(30).unwrap() );
        account.deposit(data);

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
        account.withdrawal(data);

        assert_eq!(account.available, Decimal::from_i32(70).unwrap());
        assert_eq!(account.held, Decimal::from_i32(0).unwrap());
        assert_eq!(account.total, Decimal::from_i32(90).unwrap());

        // TODO: Check overdraft failure
    }

    #[test]
    fn dispute() {
        let mut account = Account::default();

        account.available = Decimal::new(1500, 1);
        assert_eq!(account.held, Decimal::from_i32(0).unwrap());
        account.total = Decimal::from_i32(150).unwrap();
        assert_eq!(account.locked, false);

        let data = Transaction::new(1, 1, Decimal::from_i32(50).unwrap() );
        account.deposit(data);

        let data = Operation::new(1, 1);
        account.dispute(data);

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
        account.deposit(data);

        let data = Operation::new(1, 1);
        account.dispute(data);

        let data = Operation::new(1, 1);
        account.resolve(data);

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
        account.deposit(data);

        let data = Operation::new(1, 1);
        account.dispute(data);

        let data = Operation::new(1, 1);
        account.chargeback(data);

        assert_eq!(account.available, Decimal::from_i32(150).unwrap());
        assert_eq!(account.held, Decimal::from_i32(0).unwrap());
        assert_eq!(account.total, Decimal::from_i32(150).unwrap());
        assert!(account.locked);
    }
}