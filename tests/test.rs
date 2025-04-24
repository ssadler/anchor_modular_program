
use test_program::{instruction, entry};
use test_program::anchor_lang::prelude::*;


#[test]
fn test_foo_ok() {
    let data = &[1,2,3,4,5,6,7,8,5,0,0,0,0,0,0,0];
    assert_eq!(instruction::OofInstr::DISCRIMINATOR, &data[..8]);

    let account = get_account();
    entry(&test_program::ID, &[account], data).unwrap();
}

#[test]
fn test_bar_ok() {
    run_bar(3);
}


#[test]
#[should_panic]
fn test_bar_panic() {
    run_bar(0);
}


fn run_bar(n: u8) {
    let mut data = instruction::BarInstr::DISCRIMINATOR.to_vec();
    data.extend([n,0,0,0,0,0,0,0]);

    let account = get_account();
    entry(&test_program::ID, &[account], &data).unwrap();
}

const KEY: Pubkey = Pubkey::new_from_array([0;32]);
static mut LA: u64 = 0;

fn get_account<'info>() -> AccountInfo<'info> {
    #[allow(static_mut_refs)]
    AccountInfo::new(
        &KEY, true, false, unsafe { &mut LA }, &mut [], &KEY, false, 0
    )
}
