
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct FooContext<'info> {
    pub account0: UncheckedAccount<'info>
}
