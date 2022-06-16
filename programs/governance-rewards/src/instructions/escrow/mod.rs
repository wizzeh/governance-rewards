pub mod create;
pub mod transfer;

use anchor_lang::prelude::Pubkey;
pub use create::*;
pub use transfer::*;

pub fn get_escrow_owner(realm: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"escrow owner".as_ref(), realm.as_ref()], &crate::ID).0
}
