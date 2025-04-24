
use anchor_lang::prelude::*;
use super::contexts::*;


pub fn instr(ctx: Context<BarContext>, n: u64) -> Result<()> {
    assert_eq!(n, 3);
    Ok(())
}

