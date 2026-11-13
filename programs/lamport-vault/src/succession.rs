//! SPIKE ONLY: representative hook points for the `succession` feature, to measure whether the
//! default build stays byte-identical. Not the M2 implementation.
use anchor_lang::prelude::*;

use crate::{Tombstone, Vault};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Succession {
    pub claim: Pubkey,
    pub window_secs: i64,
    pub wait_secs: i64,
    pub last_signed_ts: i64,
    pub last_checkin_ts: i64,
    pub checkin_key: Pubkey,
    pub wait_started_ts: i64,
}

impl Succession {
    pub const SPACE: usize = 32 + 8 + 8 + 8 + 8 + 32 + 8;
}

impl Vault {
    pub const EXT: usize = Succession::SPACE;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct SetSuccession {
    pub claim: Pubkey,
    pub window_secs: i64,
    pub wait_secs: i64,
    pub checkin_key: Pubkey,
    pub issued_slot: u64,
    pub expiry_slot: u64,
}

#[error_code(offset = 6100)]
pub enum SuccessionError {
    #[msg("the check-in key did not sign")]
    WrongCheckinKey,
    #[msg("window or wait below the floor")]
    BelowFloor,
}

pub fn set_succession(vault: &mut Vault, s: SetSuccession) -> Result<()> {
    require!(s.window_secs >= 90 * 86_400 && s.wait_secs >= 30 * 86_400, SuccessionError::BelowFloor);
    vault.succession.claim = s.claim;
    vault.succession.window_secs = s.window_secs;
    vault.succession.wait_secs = s.wait_secs;
    vault.succession.checkin_key = s.checkin_key;
    Ok(())
}

pub fn after_signed_action(vault: &mut Vault) -> Result<()> {
    if vault.version == Tombstone::VERSION {
        return Ok(());
    }
    vault.succession.last_signed_ts = Clock::get()?.unix_timestamp;
    vault.succession.wait_started_ts = 0;
    Ok(())
}

#[derive(Accounts)]
pub struct Checkin<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    pub checkin_key: Signer<'info>,
}

pub fn handle_checkin(ctx: Context<Checkin>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    require_keys_eq!(ctx.accounts.checkin_key.key(), vault.succession.checkin_key, SuccessionError::WrongCheckinKey);
    vault.succession.last_checkin_ts = Clock::get()?.unix_timestamp;
    vault.succession.wait_started_ts = 0;
    Ok(())
}
