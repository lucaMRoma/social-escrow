use anchor_lang::prelude::*;

#[account]
pub struct EscrowAccount {
    pub amount: u64,
    pub state: EscrowState,
    pub owner: Pubkey,
    pub receiver: Pubkey,
    pub depositor: Pubkey,
}

impl EscrowAccount {
    pub const LEN: usize = 8 + 8 + 1 + 32 + 32 + 32;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum EscrowState {
    Initialized,
    Deposited,
    Accepted,
    ReleaseRequested,
    Released,
    Canceled,
}
