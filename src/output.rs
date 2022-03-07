use rust_decimal::Decimal;
use serde::Serialize;
use crate::account::Account;

#[derive(Debug, Serialize)]
pub struct Output {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

impl Output {
    pub fn convert_from(client: u16, account: Account) -> Self {
        Self {
            client,
            available: account.available(),
            held: account.held(),
            total: account.total(),
            locked: account.locked(),
        }
    }
}