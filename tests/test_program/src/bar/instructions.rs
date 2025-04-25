
use anchor_lang::prelude::*;
use super::contexts::*;


pub fn instr<'info>(ctx: Context<'_, '_, '_, 'info, BarContext<'info>>, n: u64) -> Result<()> {
    assert_eq!(n, 3);
    Ok(())
}

