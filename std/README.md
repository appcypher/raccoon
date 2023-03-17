Rename to `library/std`. There is also `library/core` for core language features that needs implementation.
The `macros` folder should also be pushed under `compiler/src/parser/macros.rs` with inline unittests.
The `std` sub crate makes no system resource assumptions apart from intrinsic access to the CPU.
Borrow ideas from Rust. GlobalAlloc.
