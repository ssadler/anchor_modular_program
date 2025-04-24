#![allow(unexpected_cfgs)]


use anchor_modular_program::*;
pub use anchor_lang::system_program::ID;
use anchor_lang::prelude::*;
pub use anchor_lang;

mod foo;
mod bar;
use foo::contexts::*;
use bar::contexts::*;



#[modular_program(modules=[foo::instructions, bar::instructions])]
pub mod big_program {
    use super::*;
}
