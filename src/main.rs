use std::{collections::HashMap, io};
use std::env;
use std::path::Path;
use account::Account;
use csv_async::{AsyncReaderBuilder, Trim};
use log::{info, debug, error};
#[cfg(test)]
use itertools::Itertools;
use smol::io::AsyncWrite;
use smol::stream::StreamExt;
use std::marker::Unpin;

mod instructions;
mod account;
mod output;
mod errors;
mod result;

use crate::result::Result;
use crate::instructions::Instruction;

#[derive(Debug, Default)]
struct Register {
    thebook: HashMap<u16, account::Account>,
}

impl Register {
    pub fn execute(&mut self, instruction: Instruction) {
        debug!("Processing account for client {}", instruction.client());
        let account = self.thebook.entry(instruction.client()).or_insert_with(|| {
            Account::default()
        });

        account.apply(instruction).unwrap_or_else(|error| {
            error!("Account instruction error: {}", error);
        })
    }

    pub async fn process(&mut self, inputfilename: &Path) -> Result {
        let input = async_fs::File::open(inputfilename).await?;
        let mut reader = AsyncReaderBuilder::new()
            .flexible(true)
            .trim(Trim::All)
            .create_deserializer(input);
    
        debug!("Consuming input data...");
        let mut records = reader.deserialize();
        while let Some(result) = records.next().await {
            let record: instructions::workaround::Instruction = result?;
            let record: Instruction = record.into();

            self.execute(record);
        }
        debug!("...consuption of input data finished.");
    
        Ok(())
    }

    pub async fn inner_dump(thebook_iter: impl IntoIterator<Item = (u16, Account)>, sink: &mut (impl AsyncWrite + Unpin)) -> Result {
        let mut writer = csv_async::AsyncSerializer::from_writer(sink);

        debug!("Dumping the book state...");
        for (client, account) in thebook_iter {
            let record = output::Output::convert_from(client, account);
            writer.serialize(record).await?
        }
        debug!("...dumping the book finished.");
        
        writer.flush().await?;
        debug!("Output writer flushed.");

        Ok(())
    }

    pub async fn dump(self, sink: &mut (impl AsyncWrite + Unpin)) -> Result {
        let thebook_iter = self.thebook.into_iter();
        Self::inner_dump(thebook_iter, sink).await
    }

    #[cfg(test)] // Outside test leave unsorted for performance reasons
    pub async fn dump_sorted(self, sink: &mut (impl AsyncWrite + Unpin)) -> Result {
        let thebook_iter = self.thebook.into_iter().sorted_by_key(|x| x.0);
        Self::inner_dump(thebook_iter, sink).await
    }
}

#[smol_potat::main]
async fn main() -> Result {
    use errors::TransactionSystemError::ArgumentsError;

    let inputfile = env::args().nth(1).ok_or_else(|| ArgumentsError("no input file provided".to_owned()))?;

    let mut register = Register::default();
    info!("Processing for {} file started.", inputfile);
    register.process(Path::new(&inputfile)).await?;
    let mut stdout = smol::Unblock::new(io::stdout());
    register.dump(&mut stdout).await?;
    info!("Processing for {} file finished.", inputfile);

    Ok(())
}

#[cfg(test)]
mod test {
    use std::io::{Write};
    use indoc::*;
    use tempfile::NamedTempFile;
    use smol::io::Cursor;

    async fn test_instructions_batch(feed: &str, expectation: &str) {
        let mut file = NamedTempFile::new().expect("failed to create temporary file");
        write!(file, "{}", feed).expect("failed to write test data");

        let mut register = super::Register::default();
        register.process(file.path()).await.expect("failed to batch process");

        let mut sink = Cursor::new(Vec::<u8>::new());
        register.dump_sorted(&mut sink).await.expect("failed to dump");

        let output: String = std::str::from_utf8(&sink.into_inner()).expect("faile to strigify the buffer").to_string();
        assert_eq!(output, expectation);
    }

    #[smol_potat::test]
    async fn basic_transactions_batch() {
        const TEST_FEED: &str = indoc!("
            type,   client, tx, amount
            deposit,     1,  1,   11.1
            deposit,     2,  2,   22.2
            deposit,     1,  3,   33.3
            deposit,     2,  4,   44.4
            deposit,     3,  5,   55.5
            withdrawal,  2,  6,   11.1
            withdrawal,  3,  7,   22.2
            withdrawal,  1,  8,   33.3
        ");

        const TEST_EXPECTATION: &str = indoc!("
            client,available,held,total,locked
            1,11.1,0,11.1,false
            2,55.5,0,55.5,false
            3,33.3,0,33.3,false
        ");

        test_instructions_batch(TEST_FEED, TEST_EXPECTATION).await
    }

    #[smol_potat::test]
    async fn dispute_operations_batch() {
        const TEST_FEED: &str = indoc!("
            type, client, tx, amount
            deposit,   1,  1, 11.1
            deposit,   2,  2, 22.2
            deposit,   1,  3, 33.3
            deposit,   2,  4, 44.4
            deposit,   3,  5, 55.5
            withdrawal,2,  6, 11.1
            withdrawal,3,  7, 22.2
            withdrawal,1,  8, 33.3
            dispute,   2,  2,
            dispute,   3,  5,
        ");

        const TEST_EXPECTATION: &str = indoc!("
            client,available,held,total,locked
            1,11.1,0,11.1,false
            2,33.3,22.2,55.5,false
            3,-22.2,55.5,33.3,false
        ");

        test_instructions_batch(TEST_FEED, TEST_EXPECTATION).await
    }

    #[smol_potat::test]
    async fn resolve_operations_batch() {
        const TEST_FEED: &str = indoc!("
            type, client,  tx,  amount
            deposit,   1,  1, 100.1234
            deposit,   2,  2, 200.2345
            deposit,   1,  3, 300.3456
            deposit,   2,  4, 400.4567
            deposit,   3,  5, 500.7891
            withdrawal,2,  6, 123.4567
            withdrawal,3,  7, 234.5678
            withdrawal,1,  8,  99.9999
            dispute,   2,  2,
            dispute,   1,  3,
            resolve,   1,  3,
            deposit,   4,  9, 600.8912
            deposit,   5, 10, 350.9123
            deposit,   5, 11, 350.1234
            dispute,   5, 10,
            resolve,   5, 10,
        ");

        const TEST_EXPECTATION: &str = indoc!("
            client,available,held,total,locked
            1,300.4691,0.0000,300.4691,false
            2,277.0000,200.2345,477.2345,false
            3,266.2213,0,266.2213,false
            4,600.8912,0,600.8912,false
            5,701.0357,0.0000,701.0357,false
        ");

        test_instructions_batch(TEST_FEED, TEST_EXPECTATION).await
    }

    #[smol_potat::test]
    async fn chargeback_operations_batch() {
        const TEST_FEED: &str = indoc!("
            type, client,  tx,  amount
            deposit,   1,  1, 100.1234
            deposit,   2,  2, 200.2345
            deposit,   1,  3, 300.3456
            deposit,   2,  4, 400.4567
            deposit,   3,  5, 500.7891
            withdrawal,2,  6, 123.4567
            withdrawal,3,  7, 234.5678
            withdrawal,1,  8,  99.9999
            dispute,   2,  2,
            dispute,   1,  3,
            chargeback,1,  3,
            deposit,   4,  9, 600.8912
            deposit,   5, 10, 350.9123
            deposit,   5, 11, 350.1234
            dispute,   5, 10,
            chargeback,5, 10,
        ");

        const TEST_EXPECTATION: &str = indoc!("
            client,available,held,total,locked
            1,0.1235,0.0000,0.1235,true
            2,277.0000,200.2345,477.2345,false
            3,266.2213,0,266.2213,false
            4,600.8912,0,600.8912,false
            5,350.1234,0.0000,350.1234,true
        ");

        test_instructions_batch(TEST_FEED, TEST_EXPECTATION).await
    }
}
