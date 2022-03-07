# Simple Transaction System

## Basics
I followed guidelines to validate in unit tests whether data format matches the specification and whether all instructions feed to the program are accepted.

## Completness
The program covers all 7 types of instructions feeding the input. The specification doesn't provide rules regarding _disputes_ and _chargebacks_ in relation to _withdrawal_ transaction, so I made assumption they **don't happen**.

## Corectness
I'm using the variants (`enum`) to distinguish transactions and operations handled, then unit tests to confirm basic logic of operations.

### Decimals
One of the biggest mistakes made by many (even experienced programmers) is attempt to use floating point numbers to keep the book. That's plain wrong as it's always possible that due to nature of mantissa and exponents values stored will loose precision. One of main benefits of the Cobol programming language is built-in out of the box support for decimals, fixed precision numbers. All financial systems should use them.

The side effect of using `Decimal` type from `rust_decimal` crate without normalization is the output containing sometimes `0.0000` for zero, but algebraically results are correct.

### Tools
The `tools/` subdirectory contains tool used to generate valid test data.

## Safety & robustness

### Stage 1: No error handling / No logging
The initial version doesn't have any error handling nor logging.

### CSV deserialization workaround
As [this issue](https://github.com/BurntSushi/rust-csv/issues/211) will remains unresolved in the `csv` crate I convert one `workaround::Instruction` type to `Instruction` type in the `input` module using `From<>` trait implementation. This may not be as efficient as direct deserialization, but with the enum type the program has architecture more pleasant to deal with.

## Efficiency

### Stage 1: Basic solution
Taking limited time I can spend on design and analysis of NFR quirks for this task I focus on solving the main problem. The obvious limitations are maintenance of scale and lack of persistence - a real life system of a similar caliber would potentially use some database in the backend.

65536 clients * 4294967296 transactions ≈ 281475G records, what means we'd need designated storage system with efficient cachce to be able to process all possible entries in this application. It's highly unlikely anyone one the planet deals with such scale at a single location.

### Stage 2: Output data
The serializable `Output` type has been used we converts account stated from, as this is simple and straightfoward solution. But it's technically possible to implement account containers, who would keep its key (in this case `client` ID) intrusively, and serialize the output directly from there. Concious of time for delivering this solution I'm not implementing it.

The function used for integration test checking actual CSV output **sorts** the output, what impacts efficiency. That's not the best solution as testability is intrusive in the code, but that's quick option to check the output; the test would need to sort the output first for comparision otherwise, what probably should be the approach for more long lasting solution in production.

## Maintainability
I firmly believe my code is self-explanatory.
