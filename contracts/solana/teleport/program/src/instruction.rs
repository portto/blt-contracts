//! Instruction types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    clock::Slot,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum TeleportInstruction {
    GetOwner,
    InitConfig,
}

pub fn get_owner(program_id: &Pubkey) -> Result<Instruction, ProgramError> {
    let init_data = TeleportInstruction::GetOwner {};
    let data = init_data.try_to_vec()?;
    let accounts = vec![];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

pub fn init_config(
    program_id: &Pubkey,
    owner: &Pubkey,
    config: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = TeleportInstruction::InitConfig {};
    let data = init_data.try_to_vec()?;
    let accounts = vec![
        AccountMeta::new(*owner, true),
        AccountMeta::new(*config, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}
