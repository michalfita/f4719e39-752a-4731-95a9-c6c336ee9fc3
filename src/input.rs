use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Transaction {
    client: u16,
    tx: u32,
    amount: Decimal,
}

impl Transaction {
    #[cfg(test)]
    pub fn new(client: u16, tx: u32, amount: Decimal) -> Self {
        Self { client, tx, amount }
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }

    pub fn client(&self) -> u16 {
        self.client
    }

    pub fn tx(&self) -> u32 {
        self.tx
    }

    pub fn negate(&mut self) {
        self.amount.set_sign_negative(true)
    }
}

#[derive(Debug, Serialize)]
pub struct Operation {
    client: u16,
    tx: u32,
}

impl Operation {
    #[cfg(test)]
    pub fn new(client: u16, tx: u32) -> Self {
        Self { client, tx }
    }

    pub fn client(&self) -> u16 {
        self.client
    }

    pub fn tx(&self) -> u32 {
        self.tx
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Instruction {
    /// A deposit is a credit to the client's asset account, meaning it should increase the available and
    /// total funds of the client account
    Deposit(Transaction),
    /// A withdraw is a debit to the client's asset account, meaning it should decrease the available and
    /// total funds of the client account. If a client does not have sufficient available funds the
    /// withdrawal should fail and the total amount of funds should not change
    Withdrawal(Transaction),
    /// A dispute represents a client's claim that a transaction was erroneous and should be reversed.
    /// The transaction shouldn't be reversed yet but the associated funds should be held. This means
    /// that the clients available funds should decrease by the amount disputed, their held funds should
    /// increase by the amount disputed, while their total funds should remain the same.
    Dispute(Operation),
    /// A resolve represents a resolution to a dispute, releasing the associated held funds. Funds that
    /// were previously disputed are no longer disputed. This means that the clients held funds should
    /// decrease by the amount no longer disputed, their available funds should increase by the
    /// amount no longer disputed, and their total funds should remain the same
    Resolve(Operation),
    /// A chargeback is the final state of a dispute and represents the client reversing a transaction.
    /// Funds that were held have now been withdrawn. This means that the clients held funds and
    /// total funds should decrease by the amount previously disputed. If a chargeback occurs the
    /// client's account should be immediately frozen.
    Chargeback(Operation),
}

impl Instruction {
    pub fn client(&self) -> u16 {
        match self {
            Instruction::Deposit(transaction) | Instruction::Withdrawal(transaction)
                => transaction.client(),
            Instruction::Dispute(operation) | Instruction::Resolve(operation) | Instruction::Chargeback(operation)
                => operation.client(),
        }
    }
}

/// Workaround for https://github.com/BurntSushi/rust-csv/issues/211
impl From<workaround::Instruction> for Instruction {
    fn from(instruction: workaround::Instruction) -> Self {
        use workaround::InstructionType as WIT;

        match instruction.typ {
            WIT::Deposit => Instruction::Deposit(Transaction{
                client: instruction.client,
                tx: instruction.tx,
                amount: instruction.amount.unwrap(),
            }),
            WIT::Withdrawal => Instruction::Withdrawal(Transaction{
                client: instruction.client,
                tx: instruction.tx,
                amount: instruction.amount.unwrap(),
            }),
            WIT::Dispute => Instruction::Dispute(Operation{
                client: instruction.client,
                tx: instruction.tx,
            }),
            WIT::Resolve => Instruction::Resolve(Operation{
                client: instruction.client,
                tx: instruction.tx,
            }),
            WIT::Chargeback => Instruction::Chargeback(Operation{
                client: instruction.client,
                tx: instruction.tx,
            }),
        }
        
    }
}

pub mod workaround {
    use rust_decimal::Decimal;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "lowercase")]
    pub enum InstructionType {
        Deposit,
        Withdrawal,
        Dispute,
        Resolve,
        Chargeback,
    }

    #[derive(Deserialize, Debug)]
    pub struct Instruction {
        #[serde(rename = "type")]
        pub (super) typ: InstructionType,
        pub (super) client: u16,
        pub (super) tx: u32,
        pub (super) amount: Option<Decimal>,
    }
}
