# Simple Transaction System

## Basics
I followed guidelines to validate in unit tests whether data format matches the specification and whether all instructions feed to the program are accepted.

## Completness
The program covers all 7 types of instructions feeding the input. The specification doesn't provide rules regarding _disputes_ and _chargebacks_ in relation to _withdrawal_ transaction, so I made assumption they **don't happen**.

Please refer to the [Specification flaw](#specification-flaw) subsection below for more details.

From production quality perspective the application has proper error handling and logging.

## Corectness
I'm using the variants (`enum`) to distinguish transactions and operations handled, then unit tests to confirm basic logic of operations.

### Decimals
One of the biggest mistakes made by many (even experienced programmers) is attempt to use floating point numbers to keep the book. That's plain wrong as it's always possible that due to nature of mantissa and exponents values stored will loose precision. One of main benefits of the Cobol programming language is built-in out of the box support for decimals, fixed precision numbers. All financial systems should use them.

The side effect of using `Decimal` type from `rust_decimal` crate without normalization is the output containing sometimes `0.0000` for zero, but algebraically results are correct.

### Tools
The `tools/` subdirectory contains tool used to generate valid test data.

### Specification flaw
The specification of `dispute` and `chargeback` dispositions doesn't state any rules regarding these putting the _available_ funds in overdraft (below 0). This implementation allows this to happen, but the specification should correctly define the expected behaviour, for example in case after certain deposits there was whithdrawal of higher sum than required for disputes later.

The specification of `whithdrawal`, `dispute` and `chargeback` do not mention them together, what makes it open for interpretation; the most straightforward one is to assume it doesn't happen in the input data. I understand this may be plain wrong.

The specification of `dispute`, `resolve` and `chargeback` do not mention how to deal with multiple attempts of these instructions for the same transaction, epecially for cases like for example multiple disputes before `resolve` or `chargeback`. I changed my initial assumption and implemented (_Stage 5_) the mechanism holding transations' state and acting sensible; it prevents multiple resolutions and multiple chagebacks and allow both only for already disputed transactions, plus new dispute for resolved transaction.

In normal work conditions both above flaws would be raised for clarification with departament or people responsible for preparing the document in the first place.

## Safety & robustness

### Stage 1: No error handling / No logging
The initial version doesn't have any error handling nor logging.

### CSV deserialization workaround
As [this issue](https://github.com/BurntSushi/rust-csv/issues/211) will remains unresolved in the `csv` crate I convert one `workaround::Instruction` type to `Instruction` type in the `input` module using `From<>` trait implementation. This may not be as efficient as direct deserialization, but with the enum type the program has architecture more pleasant to deal with.

### Stage 3: Proper error handling
The most commonly used `thiserror` crate is harnessed to create error structure with error variants used to utilize with `Result<T,E>` as result type in functions in the application. Please refer to `src/error.rs` for details.

### Stage 4: Logging
Added logging can be enabled and used to diagnose problems with the program.

## Efficiency

### Stage 1: Basic solution
Taking limited time I can spend on design and analysis of NFR quirks for this task I focus on solving the main problem. The obvious limitations are maintenance of scale and lack of persistence - a real life system of a similar caliber would potentially use some database in the backend.

65536 clients * 4294967296 transactions ≈ 281475G records, what means we'd need designated storage system with efficient cachce to be able to process all possible entries in this application. It's highly unlikely anyone one the planet deals with such scale at a single location.

### Stage 2: Output data
The serializable `Output` type has been used we converts account stated from, as this is simple and straightfoward solution. But it's technically possible to implement account containers, who would keep its key (in this case `client` ID) intrusively, and serialize the output directly from there. Concious of time for delivering this solution I'm not implementing it.

The function used for integration test checking actual CSV output **sorts** the output, what impacts efficiency. That's not the best solution as testability is intrusive in the code, but that's quick option to check the output; the test would need to sort the output first for comparision otherwise, what probably should be the approach for more long lasting solution in production.

### Stage 3: Error handling
Adding error handling to `account.rs` has impact on processing power in cases when there are errors in input data. This is demonstration bringing the code closer to the production system, even if they're ignored as specification states, but it opens door for get them logged.

### Stage 4: Logging
Adding logging if not disabled may impact the processing efficiency. However, this as feature is disabled.

## Maintainability
I firmly believe my code is self-explanatory.
