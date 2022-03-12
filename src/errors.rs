use thiserror::Error;
use csv_async::Error as CSVError;
use csv::Error as LegacyCSVError;
use crate::instructions::{Transaction, Operation, TransactionState};
use std::io::Error as IOError;

#[derive(Error, Debug)]
pub enum TransactionSystemError {
    #[error("Arguments error")]
    ArgumentsError(String),
    #[error("CSV processing failure")]
    CSVError(#[from] CSVError),
    #[error("CSV processing failure")]
    LegacyCSVError(#[from] LegacyCSVError),
    #[error("I/O operation failure")]
    IOError(#[from] IOError),
    #[error("Transaction processing failure: {message} / {transaction:?}")]
    TransactionError {
        message: String,
        transaction: Transaction,
    },
    #[error("Operation executing failure: {message} / {operation:?}")]
    OperationError {
        message: String,
        operation: Operation,
    },
    #[error("Illegal attempt to change state: {oldstate} => {newstate}")]
    TransactionStateError {
        oldstate: TransactionState,
        newstate: TransactionState
    }
}