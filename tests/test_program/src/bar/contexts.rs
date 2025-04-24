
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct BarContext<'info> {
    pub account0: UncheckedAccount<'info>
}
