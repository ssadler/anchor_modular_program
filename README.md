# Anchor Modular Program

Replacement `#[program]` macro that allows specifying additional instruction modules

## Usage

Lets say you want to import instructions from your module `extra::instructions`,
and you have the required types (contexts, instruction argument types) at
`extra::types`:

```rust

use anchor_lang::prelude::*;
use anchor_modular_program::*;

declare_id!("...");

mod extra;
use extra::types::*;

#[modular_program(
    modules=[
        extra::instructions
    ]
)]
mod my_program {
    use super::*;
}
```

Instructions from `extra::instructions` will be included (forwarded, rather than
included directly), prefixed with `extra_`.

## Overrides

As well as specifying a module path in the local program, you can provide a spec:

```rust
#[modular_program(
    modules=[
        {
            module: etc,                  // rust module path to instructions module
            prefix: "etc",                // prefixed with "etc_", or "" for no prefix
            file_path: "./src/etc/mod.rs" // path to instructions
        }
    ]
)
```

## Examples

See the [Test Program](/tests/test_program/src/lib.rs) for examples.

## How it works

1. In the above example, `extra::instructions` is converted to the file path
   `src/extra/instructions.rs`.
2. The file is read and parsed, using anchor's own Program parsing.
3. Relay instructions are created, and appended to the TokenStream of the
   main program module.
4. The final program is built from the modified TokenStream.

## Contributing

PRs welcome
