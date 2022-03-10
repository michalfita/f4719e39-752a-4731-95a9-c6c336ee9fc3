use std::io::Write;
use std::{collections::HashMap, io};
use std::env;
use std::path::Path;
use account::Account;
use csv::{ReaderBuilder, Trim};
#[cfg(test)]
use itertools::Itertools;
use log::{info, debug, error};

mod input;
mod account;
mod output;
mod errors;
mod result;

use crate::result::Result;

#[derive(Debug, Default)]
struct Register {
    thebook: HashMap<u16, account::Account>,
}

impl Register {
    pub fn execute(&mut self, instruction: input::Instruction) {
        debug!("Processing account for client {}", instruction.client());
        let account = self.thebook.entry(instruction.client()).or_insert_with(|| {
            Account::default()
        });

        account.apply(instruction).unwrap_or_else(|error| {
            error!("Account instruction error: {}", error);
        })
    }

    pub fn process(&mut self, inputfilename: &Path) -> Result {
        let mut reader = ReaderBuilder::new()
            .flexible(true)
            .trim(Trim::All)
            .from_path(inputfilename)?;
    
        debug!("Consuming input data...");
        for result in reader.deserialize() {
            let record: input::workaround::Instruction = result?;
            let record: input::Instruction = record.into();

            self.execute(record);
        }
        debug!("...consuption of input data finished.");
    
        Ok(())
    }

    fn inner_dump(thebook_iter: impl IntoIterator<Item = (u16, Account)>, sink: &mut impl Write) -> Result {
        let mut writer = csv::Writer::from_writer(sink);

        debug!("Dumping the book state...");
        for (client, account) in thebook_iter {
            let record = output::Output::convert_from(client, account);
            writer.serialize(record)?
        }
        debug!("...dumping the book finished.");
        
        writer.flush()?;
        debug!("Output writer flushed.");

        Ok(())
    }

    pub fn dump(self, sink: &mut impl Write) -> Result {
        let thebook_iter = self.thebook.into_iter();
        Self::inner_dump(thebook_iter, sink)
    }

    #[cfg(test)] // Outside test leave unsorted for performance reasons
    pub fn dump_sorted(self, sink: &mut impl Write) -> Result {
        let thebook_iter = self.thebook.into_iter().sorted_by_key(|x| x.0);
        Self::inner_dump(thebook_iter, sink)
    }
}

fn main() -> Result {
    use errors::TransactionSystemError::ArgumentsError;

    let inputfile = env::args().nth(1).ok_or_else(|| ArgumentsError("no input file provided".to_owned()))?;

    let mut register = Register::default();
    info!("Processing for {} file started.", inputfile);
    register.process(Path::new(&inputfile))?;
    register.dump(&mut io::stdout())?;
    info!("Processing for {} file finished.", inputfile);

    Ok(())
}

#[cfg(test)]
mod test {
    use std::io::{Write, self};
    use indoc::*;
    use tempfile::NamedTempFile;

    fn test_instructions_batch(feed: &str, expectation: &str) {
        let mut file = NamedTempFile::new().expect("failed to create temporary file");
        write!(file, "{}", feed).expect("failed to write test data");

        let mut register = super::Register::default();
        register.process(file.path()).expect("failed to batch process");

        let mut sink = io::Cursor::new(Vec::<u8>::new());
        register.dump_sorted(&mut sink).expect("failed to dump");

        let output: String = std::str::from_utf8(&sink.into_inner()).expect("faile to strigify the buffer").to_string();
        assert_eq!(output, expectation);
    }

    #[test]
    fn basic_transactions_batch() {
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

        test_instructions_batch(TEST_FEED, TEST_EXPECTATION)
    }

    #[test]
    fn dispute_operations_batch() {
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

        test_instructions_batch(TEST_FEED, TEST_EXPECTATION)
    }

    #[test]
    fn resolve_operations_batch() {
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

        test_instructions_batch(TEST_FEED, TEST_EXPECTATION)
    }

    #[test]
    fn chargeback_operations_batch() {
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

        test_instructions_batch(TEST_FEED, TEST_EXPECTATION)
    }
}
