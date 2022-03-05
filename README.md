# Simple Transaction System

## Corectness
I'm using the variants (`enum`) to distinguish transactions and operations handled, then unit tests to confirm basic logic of operations.

### Decimals
One of the biggest mistakes made by many (even experienced programmers) is attempt to use floating point numbers to keep the book. That's plain wrong as it's always possible that due to nature of mantissa and exponents values stored will loose precision. One of main benefits of the Cobol programming language is built-in out of the box support for decimals, fixed precision numbers. All financial systems should use them.

## Safety & robustness

### Stage 1: No error handling / No logging
The initial version doesn't have any error handling nor logging.

## Efficiency

### Stage 1: Basic solution
Taking limited time I can spend on design and analysis of NFR quirks for this task I focus on solving the main problem. The obvious limitations are maintenance of scale and lack of persistence - a real life system of a similar caliber would potentially use some database in the backend.

65536 clients * 4294967296 transactions ≈ 281475G records, what means we'd need designated storage system with efficient cachce to be able to process all possible entries in this application. It's highly unlikely anyone one the planet deals with such scale at a single location.

## Maintainability
I firmly believe my code is self-explanatory.
