#![allow(unexpected_cfgs)]

pub use anchor_lang::system_program::ID;
pub use anchor_lang;

use anchor_modular_program::*;
use anchor_lang::prelude::*;


macro_rules! foo_wrapper {
    ($ix:path, $ctx:ident: $ctx_type:ty $(, $arg:ident: $arg_type:ty )*) => {
        { msg!("Foo called"); $ix($ctx $(, $arg << 1)*) }
    };
}


mod foo;
mod bar;
use foo::contexts::*;
use bar::contexts::*;

#[modular_program(modules=[
    bar::instructions,
    {
        module: foo,
        file_path: "src/foo/mod.rs",
        prefix: "oof",
        wrapper: foo_wrapper
    }
])]
pub mod big_program {
    use super::*;
}
