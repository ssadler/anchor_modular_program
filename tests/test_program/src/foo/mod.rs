
pub mod contexts;


use anchor_lang::prelude::*;
use contexts::*;


#[instruction(discriminator = &[1,2,3,4,5,6,7,8])]
pub fn instr(ctx: Context<FooContext>, n: u64) -> Result<()> {
    assert_eq!(n, 10);
    Ok(())
}

