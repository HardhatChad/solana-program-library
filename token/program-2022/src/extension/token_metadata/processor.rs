//! Token-metadata processor

use {
    crate::{
        check_program_account,
        error::TokenError,
        extension::{
            alloc_and_serialize, metadata_pointer::MetadataPointer, BaseStateWithExtensions,
            StateWithExtensions,
        },
        state::Mint,
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program_error::ProgramError,
        program_option::COption,
        pubkey::Pubkey,
    },
    spl_token_metadata_interface::{
        error::TokenMetadataError,
        instruction::{
            Emit, Initialize, RemoveKey, TokenMetadataInstruction, UpdateAuthority, UpdateField,
        },
        state::{OptionalNonZeroPubkey, TokenMetadata},
    },
};

/// Processes a [Initialize](enum.TokenMetadataInstruction.html) instruction.
pub fn process_initialize(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: Initialize,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let metadata_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;

    // check that the mint and metadata accounts are the same, since the metadata
    // extension should only describe itself
    if metadata_info.key != mint_info.key {
        msg!("Metadata for a mint must be initialized in the mint itself.");
        return Err(TokenError::MintMismatch.into());
    }

    // scope the mint authority check, since the mint is in the same account!
    {
        // This check isn't really needed since we'll be writing into the account,
        // but auditors like it
        check_program_account(mint_info.owner)?;
        let mint_data = mint_info.try_borrow_data()?;
        let mint = StateWithExtensions::<Mint>::unpack(&mint_data)?;

        if !mint_authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if mint.base.mint_authority.as_ref() != COption::Some(mint_authority_info.key) {
            return Err(TokenMetadataError::IncorrectMintAuthority.into());
        }

        if mint.get_extension::<MetadataPointer>().is_err() {
            msg!("A mint with metadata must have the metadata-pointer extension initialized");
            return Err(TokenError::InvalidExtensionCombination.into());
        }
    }

    // Create the token metadata
    let update_authority = OptionalNonZeroPubkey::try_from(Some(*update_authority_info.key))?;
    let token_metadata = TokenMetadata {
        name: data.name,
        symbol: data.symbol,
        uri: data.uri,
        update_authority,
        mint: *mint_info.key,
        ..Default::default()
    };

    // allocate a TLV entry for the space and write it in, assumes that there's
    // enough SOL for the new rent-exemption
    alloc_and_serialize::<Mint, _>(metadata_info, &token_metadata, false)?;

    Ok(())
}

/// Processes an [UpdateField](enum.TokenMetadataInstruction.html) instruction.
pub fn process_update_field(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _data: UpdateField,
) -> ProgramResult {
    Ok(())
}

/// Processes a [RemoveKey](enum.TokenMetadataInstruction.html) instruction.
pub fn process_remove_key(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _data: RemoveKey,
) -> ProgramResult {
    Ok(())
}

/// Processes a [UpdateAuthority](enum.TokenMetadataInstruction.html) instruction.
pub fn process_update_authority(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _data: UpdateAuthority,
) -> ProgramResult {
    Ok(())
}

/// Processes an [Emit](enum.TokenMetadataInstruction.html) instruction.
pub fn process_emit(_program_id: &Pubkey, _accounts: &[AccountInfo], _data: Emit) -> ProgramResult {
    Ok(())
}

/// Processes an [Instruction](enum.Instruction.html).
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: TokenMetadataInstruction,
) -> ProgramResult {
    match instruction {
        TokenMetadataInstruction::Initialize(data) => {
            msg!("TokenMetadataInstruction: Initialize");
            process_initialize(program_id, accounts, data)
        }
        TokenMetadataInstruction::UpdateField(data) => {
            msg!("TokenMetadataInstruction: UpdateField");
            process_update_field(program_id, accounts, data)
        }
        TokenMetadataInstruction::RemoveKey(data) => {
            msg!("TokenMetadataInstruction: RemoveKey");
            process_remove_key(program_id, accounts, data)
        }
        TokenMetadataInstruction::UpdateAuthority(data) => {
            msg!("TokenMetadataInstruction: UpdateAuthority");
            process_update_authority(program_id, accounts, data)
        }
        TokenMetadataInstruction::Emit(data) => {
            msg!("TokenMetadataInstruction: Emit");
            process_emit(program_id, accounts, data)
        }
    }
}