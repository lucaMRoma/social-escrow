use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};
use anchor_lang::solana_program::rent::Rent;

use crate::state::*;
use crate::error::EscrowError;
use crate::contexts::*;

pub fn initialize(ctx: Context<Initialize>, amount: u64, owner: Pubkey, receiver: Pubkey) -> Result<()> {
    require!(amount > 0, EscrowError::InvalidAmount);

    // Calculate rent-exempt balance using the correct method for your Solana version
    let rent_exempt_balance = Rent::get()?.minimum_balance(EscrowAccount::LEN);

    // Ensure the user has sufficient funds to cover the escrow amount and rent
    require!(
        **ctx.accounts.user.lamports.borrow() >= amount + rent_exempt_balance,
        EscrowError::InsufficientFunds
    );

    // Create a transfer instruction to transfer funds from the user to the escrow account
    let transfer_instruction = system_instruction::transfer(
        &ctx.accounts.user.key(),
        &ctx.accounts.escrow_account.key(),
        amount,
    );

    // Invoke the transfer instruction, passing the necessary accounts
    invoke(
        &transfer_instruction,
        &[ctx.accounts.user.to_account_info(), ctx.accounts.escrow_account.to_account_info()],
    )?;

    // Borrow the escrow account mutably for modification
    let escrow_account = &mut ctx.accounts.escrow_account;

    // Initialize the escrow account data
    escrow_account.amount = amount;
    escrow_account.state = EscrowState::Initialized;
    escrow_account.owner = owner;
    escrow_account.receiver = receiver;
    escrow_account.depositor = *ctx.accounts.user.key;
    escrow_account.state = EscrowState::Deposited; // Update state to 'Deposited'

    Ok(())
}


pub fn accept(ctx: Context<Accept>) -> Result<()> {
    let escrow_account = &mut ctx.accounts.escrow_account;
    
    // Verifica che l'utente sia il ricevente dell'escrow account
    require!(*ctx.accounts.user.key == escrow_account.receiver, EscrowError::Unauthorized);
    
    // Verifica che lo stato dell'escrow account sia 'Deposited'
    require!(escrow_account.state == EscrowState::Deposited, EscrowError::InvalidState);
    
    // Aggiorna lo stato a 'Accepted'
    escrow_account.state = EscrowState::Accepted;
    
    Ok(())
}

pub fn request_release(ctx: Context<RequestRelease>) -> Result<()> {
    let escrow_account = &mut ctx.accounts.escrow_account;
    
    // Verifica che l'utente sia il ricevente dell'escrow account
    require!(*ctx.accounts.user.key == escrow_account.receiver, EscrowError::Unauthorized);
    
    // Verifica che lo stato dell'escrow account sia 'Accepted'
    require!(escrow_account.state == EscrowState::Accepted, EscrowError::InvalidState);
    
    // Aggiorna lo stato a 'ReleaseRequested'
    escrow_account.state = EscrowState::ReleaseRequested;
    
    Ok(())
}


pub fn release(ctx: Context<Release>) -> Result<()> {
    let escrow_account_info = ctx.accounts.escrow_account.to_account_info();
    let user_key = ctx.accounts.user.key();
    let depositor_key = ctx.accounts.escrow_account.depositor; // Chi ha depositato i fondi
    let escrow_state = &ctx.accounts.escrow_account.state; // Prendiamo in prestito il valore per immutabilità
    let escrow_amount = ctx.accounts.escrow_account.amount;
    let receiver_key = ctx.accounts.escrow_account.receiver;
    let owner_info = ctx.accounts.owner.to_account_info();
    let system_program_info = ctx.accounts.system_program.to_account_info();
    let receiver_info = ctx.accounts.receiver.to_account_info();

    // Verifica che l'utente sia il depositante dell'escrow account
    require!(user_key == depositor_key, EscrowError::Unauthorized);

    // Verifica che lo stato dell'escrow account sia 'ReleaseRequested'
    require!(*escrow_state == EscrowState::ReleaseRequested, EscrowError::InvalidState);

    // Calcolo dell'importo da trasferire e della fee
    let fee = escrow_amount / 100; // 1% fee
    let receiver_amount = escrow_amount - fee;

    // Costruzione e invocazione della transazione per il trasferimento della fee
    let transfer_fee_instruction = system_instruction::transfer(
        &escrow_account_info.key,
        &ctx.accounts.owner.key(), // Proprietario della piattaforma
        fee,
    );
    invoke(
        &transfer_fee_instruction,
        &[
            escrow_account_info.clone(),
            owner_info.clone(),
            system_program_info.clone(),
        ],
    )?;

    // Costruzione e invocazione della transazione per il trasferimento al receiver
    let transfer_receiver_instruction = system_instruction::transfer(
        &escrow_account_info.key,
        &receiver_key,
        receiver_amount,
    );
    invoke(
        &transfer_receiver_instruction,
        &[
            escrow_account_info.clone(),
            receiver_info.clone(),
            system_program_info.clone(),
        ],
    )?;

    // Aggiornamento dello stato dell'escrow account
    let escrow_account = &mut ctx.accounts.escrow_account;
    escrow_account.state = EscrowState::Released;

    Ok(())
}


pub fn cancel(ctx: Context<Cancel>) -> Result<()> {
    let escrow_account_info = ctx.accounts.escrow_account.to_account_info();
    let user_key = ctx.accounts.user.key();
    let escrow_owner = ctx.accounts.escrow_account.owner;
    let escrow_receiver = ctx.accounts.escrow_account.receiver;
    let escrow_depositor = ctx.accounts.escrow_account.depositor;
    let escrow_state = &ctx.accounts.escrow_account.state; // Prendiamo in prestito il valore per immutabilità
    let escrow_amount = ctx.accounts.escrow_account.amount;
    let depositor_info = ctx.accounts.depositor.to_account_info();
    let system_program_info = ctx.accounts.system_program.to_account_info();

    // Verifica dell'autorizzazione
    require!(
        user_key == escrow_owner
            || (user_key == escrow_receiver
                && (*escrow_state == EscrowState::Deposited
                    || *escrow_state == EscrowState::Accepted))
            || (user_key == escrow_depositor && *escrow_state == EscrowState::Deposited),
        EscrowError::Unauthorized
    );

    // Verifica che lo stato dell'escrow account non sia 'Released'
    require!(*escrow_state != EscrowState::Released, EscrowError::InvalidState);

    // Costruzione e invocazione della transazione per il trasferimento al depositante
    let transfer_instruction = system_instruction::transfer(
        &escrow_account_info.key,
        &escrow_depositor,
        escrow_amount,
    );
    invoke(
        &transfer_instruction,
        &[
            escrow_account_info.clone(),
            depositor_info.clone(),
            system_program_info.clone(),
        ],
    )?;

    // Aggiornamento dello stato dell'escrow account
    let escrow_account = &mut ctx.accounts.escrow_account;
    escrow_account.state = EscrowState::Canceled;

    Ok(())
}
