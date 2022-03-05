use std::collections::HashMap;
use std::env;
use std::path::Path;
use account::Account;
use csv::{ReaderBuilder, Trim};

mod input;
mod account;

#[derive(Debug, Default)]
struct Register {
    thebook: HashMap<u16, account::Account>,
}

impl Register {
    pub fn execute(&mut self, instruction: input::Instruction) {
        
        let account = self.thebook.entry(instruction.client()).or_insert_with(|| {
            Account::default()
        });

        account.apply(instruction);
    }

    pub fn process(&mut self, inputfilename: &Path) -> csv::Result<()> {
        let mut reader = ReaderBuilder::new()
            .flexible(true)
            .trim(Trim::All)
            .from_path(inputfilename)
            .expect("failed to open the input file");
    
        for result in reader.deserialize() {
            let record: input::workaround::Instruction = result?;
            let record: input::Instruction = record.into();

            self.execute(record);
        }
    
        Ok(())
    }
}

fn main() {
    let inputfile = env::args().nth(1).expect("no input file");
   
    let mut register = Register::default();
    register.process(Path::new(&inputfile)).expect("processing failed");
    //let elem = input::Instruction::Deposit(input::Transaction{client: 3, tx: 5, amount: Decimal::new(45, 1)});
    //let data: Vec<Instruction> = vec![elem];

    //let mut wtr = csv::Writer::from_writer(io::stdout());

    // wtr.serialize(elem).expect("failed to serialize");
    // wtr.flush().expect("failed to flush");
}

#[cfg(test)]
mod test {
    use std::io::Write;
    use indoc::*;
    use tempfile::NamedTempFile;

    #[test]
    fn basic_transactions_batch() {
        const TEST_DATA: &str = indoc!("
            type,    client, tx, amount
            deposit,      1,  1,    1.0
            deposit,      2,  2,    2.0
            deposit,      1,  3,    2.0
            withdrawal,   1,  4,    1.5
            withdrawal,   1,  5,    3.0
        ");

        let mut file = NamedTempFile::new().expect("failed to create temporary file");
        write!(file, "{}", TEST_DATA).expect("failed to write test data");

        let mut register = super::Register::default();
        register.process(file.path()).expect("failed to batch process");
    }
}