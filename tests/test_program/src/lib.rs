#![allow(unexpected_cfgs)]

pub use anchor_lang::system_program::ID;
pub use anchor_lang;

use anchor_modular_program::*;
use anchor_lang::prelude::*;


mod foo;
mod bar;
use foo::contexts::*;
use foo::instrocshons as baz;
use bar::contexts::*;

#[modular_program(modules=[
    bar::instructions,
    { module: baz, file_path: "src/foo/instrocshons.rs", prefix: "oof" }
])]
pub mod big_program {
    use super::*;
}
