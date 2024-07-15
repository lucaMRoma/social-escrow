use anchor_lang::prelude::*;

pub mod processor;
pub mod state;
pub mod contexts;
pub mod error;

use crate::contexts::{Cancel, Initialize, Accept, RequestRelease, Release};

declare_id!("BbTM44iV6i7Bxbu7pfK5gWWAs8fk8ebQ27gTDmz22rtD");

#[program]
mod escrow {
    use super::*;

    #[inline(always)]
    pub fn initialize(ctx: Context<Initialize>, amount: u64, owner: Pubkey, receiver: Pubkey) -> Result<()> {
        processor::initialize(ctx, amount, owner, receiver)
    }

    #[inline(always)]
    pub fn accept(ctx: Context<Accept>) -> Result<()> {
        processor::accept(ctx)
    }

    #[inline(always)]
    pub fn request_release(ctx: Context<RequestRelease>) -> Result<()> {
        processor::request_release(ctx)
    }

    #[inline(always)]
    pub fn release(ctx: Context<Release>) -> Result<()> {
        processor::release(ctx)
    }

    #[inline(always)]
    pub fn cancel(ctx: Context<Cancel>) -> Result<()> {
        processor::cancel(ctx)
    }
}
