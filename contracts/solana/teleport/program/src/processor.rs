//! Program state processor

use {
    crate::{error::TeleportError, instruction::TeleportInstruction, state},
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::next_account_info,
        account_info::AccountInfo,
        entrypoint::ProgramResult,
        instruction::{AccountMeta, Instruction},
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        pubkey::Pubkey,
        rent::Rent,
        system_instruction, system_program,
        sysvar::Sysvar,
    },
    spl_token,
    std::str::FromStr,
};

/// Program state handler.
pub struct Processor {}
impl Processor {
    /// Processes an instruction
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        input: &[u8],
    ) -> ProgramResult {
        let instruction = TeleportInstruction::try_from_slice(input)?;
        match instruction {
            TeleportInstruction::GetOwner => {
                msg!("Instruction: GetOwner");
                let owner = Pubkey::from_str(state::OWNER_KEY).unwrap();
                msg!(&format!("owner is {}", owner.to_string()));
                Ok(())
            }
            TeleportInstruction::InitConfig => {
                msg!("Instruction: InitConfig");
                Self::process_init_config(program_id, accounts)
            }
            TeleportInstruction::InitAdmin { auth, allowance } => {
                msg!("Instruction: InitAdmin");
                Self::process_init_admin(program_id, accounts, &auth, allowance)
            }
            TeleportInstruction::InitTeleportOutRecord => {
                msg!("Instruction: InitTeleportOutRecord");
                Self::process_init_teleport_out_record(program_id, accounts)
            }
            TeleportInstruction::AddAdmin { admin } => {
                msg!("Instruction: AddAdmin");
                Self::process_add_admin(program_id, accounts, &admin)
            }
            TeleportInstruction::RemoveAdmin { admin } => {
                msg!("Instruction: RemoveAdmin");
                Self::process_remove_admin(program_id, accounts, &admin)
            }
            TeleportInstruction::Freeze => {
                msg!("Instruction: Freeze");
                Self::process_freeze(program_id, accounts)
            }
            TeleportInstruction::Unfreeze => {
                msg!("Instruction: Unfreeze");
                Self::process_unfreeze(program_id, accounts)
            }
            TeleportInstruction::TeleportIn {
                amount,
                decimals,
                to: _,
            } => {
                msg!("Instruction: TeleportIn");
                Self::process_teleport_in(program_id, accounts, amount, decimals)
            }
            TeleportInstruction::TeleportOut {
                tx_hash,
                amount,
                decimals,
            } => {
                msg!("Instruction: TeleportOut");
                Self::process_teleport_out(program_id, accounts, &tx_hash, amount, decimals)
            }
            TeleportInstruction::DepositAllowance { allowance } => {
                msg!("Instruction: DepositAllowance");
                Self::process_deposit_allowance(program_id, accounts, allowance)
            }
            TeleportInstruction::CloseTeleportOutRecord {} => {
                msg!("Instruction: TeleportInstruction");
                Self::process_close_teleport_out_record(program_id, accounts)
            }
        }
    }

    pub fn process_init_config(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let owner_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_info)?;

        Self::only_owner(owner_info)?;

        let mut config = state::Config::try_from_slice(&config_info.data.borrow())?;
        if config.is_init {
            return Err(TeleportError::AlreadyInUse.into());
        }
        if !rent.is_exempt(config_info.lamports(), config_info.data_len()) {
            return Err(TeleportError::NotRentExempt.into());
        }

        config.is_init = true;

        config
            .serialize(&mut *config_info.data.borrow_mut())
            .map_err(|e| e.into())
    }

    pub fn process_init_admin(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        auth: &Pubkey,
        allowance: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let owner_info = next_account_info(account_info_iter)?;
        let admin_info = next_account_info(account_info_iter)?;

        Self::only_owner(owner_info)?;

        let mut admin = state::Admin::try_from_slice(&admin_info.data.borrow())?;
        if admin.is_init {
            return Err(TeleportError::AlreadyInUse.into());
        }

        admin.is_init = true;
        admin.auth = *auth;
        admin.allowance = allowance;

        admin
            .serialize(&mut *admin_info.data.borrow_mut())
            .map_err(|e| e.into())
    }

    pub fn process_init_teleport_out_record(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let record_info = next_account_info(account_info_iter)?;

        let mut record = state::TeleportOutRecord::try_from_slice(&record_info.data.borrow())?;
        if record.is_init {
            return Err(TeleportError::AlreadyInUse.into());
        }

        record.is_init = true;
        record
            .serialize(&mut *record_info.data.borrow_mut())
            .map_err(|e| e.into())
    }

    pub fn process_add_admin(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        admin: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let owner_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;

        Self::only_owner(owner_info)?;

        let mut config = state::Config::try_from_slice(&config_info.data.borrow())?;
        if !config.is_init {
            return Err(TeleportError::IncorrectProgramAccount.into());
        }

        config.add_admin(admin)?;

        config
            .serialize(&mut *config_info.data.borrow_mut())
            .map_err(|e| e.into())
    }

    pub fn process_remove_admin(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        admin: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let owner_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;

        Self::only_owner(owner_info)?;

        let mut config = state::Config::try_from_slice(&config_info.data.borrow())?;
        if !config.is_init {
            return Err(TeleportError::IncorrectProgramAccount.into());
        }

        config.remove_admin(admin)?;

        config
            .serialize(&mut *config_info.data.borrow_mut())
            .map_err(|e| e.into())
    }

    pub fn process_freeze(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let owner_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;

        Self::only_owner(owner_info)?;

        let mut config = state::Config::try_from_slice(&config_info.data.borrow())?;
        if !config.is_init {
            return Err(TeleportError::IncorrectProgramAccount.into());
        }

        config.is_frozen = true;

        config
            .serialize(&mut *config_info.data.borrow_mut())
            .map_err(|e| e.into())
    }

    pub fn process_unfreeze(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let owner_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;

        Self::only_owner(owner_info)?;

        let mut config = state::Config::try_from_slice(&config_info.data.borrow())?;
        if !config.is_init {
            return Err(TeleportError::IncorrectProgramAccount.into());
        }

        config.is_frozen = false;

        config
            .serialize(&mut *config_info.data.borrow_mut())
            .map_err(|e| e.into())
    }

    pub fn process_teleport_in(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        decimals: u8,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let config_info = next_account_info(account_info_iter)?;
        let wallet_info = next_account_info(account_info_iter)?;
        let wallet_pda_info = next_account_info(account_info_iter)?;
        let wallet_signer_info = next_account_info(account_info_iter)?;
        let wallet_program_info = next_account_info(account_info_iter)?;
        let from_info = next_account_info(account_info_iter)?;
        let mint_info = next_account_info(account_info_iter)?;
        let from_auth_info = next_account_info(account_info_iter)?;
        let spl_token_program_info = next_account_info(account_info_iter)?;

        let config = Self::get_config(program_id, config_info)?;
        if config.is_frozen {
            return Err(TeleportError::Freeze.into());
        }

        // check wallet program
        let expected_multisig_program = Pubkey::from_str(state::MULTISIG_PROGRAM_KEY).unwrap();
        if wallet_program_info.key != &expected_multisig_program{
            msg!("unexpected multisig program");
            return Err(TeleportError::UnexpectedError.into());
        }

        // check token program
        if spl_token_program_info.key != &spl_token::id() {
            msg!("unexpected spl-token-program");
            return Err(TeleportError::UnexpectedError.into());
        }

        // check mint
        let expected_blt = Pubkey::from_str(state::BLT_MINT_KEY).unwrap();
        if mint_info.key != &expected_blt {
            msg!("unexpected mint");
            return Err(TeleportError::UnexpectedError.into());
        }

        let seeds: &[&[_]] = &[
            state::SIGNER_SEED,
            &[Pubkey::find_program_address(&[state::SIGNER_SEED], &program_id).1],
        ];

        let mut data = vec![
            3, // u8, wallet program invoke instruction
            2, // u8, invoke program idx
            3, 0, // u16, total account, little endian
            4, 1, // u8, u8, account idx, not signer writable
            5, 1, // ..
            6, 2,  // readonly singer
            15, // u8, mint instruction in token program
        ];
        data.extend(amount.to_le_bytes().iter().cloned());
        data.push(decimals);

        invoke_signed(
            &Instruction::new_with_bytes(
                *wallet_program_info.key,
                &data[..],
                vec![
                    AccountMeta::new(*wallet_info.key, false),
                    AccountMeta::new_readonly(*wallet_pda_info.key, false),
                    AccountMeta::new_readonly(*spl_token_program_info.key, false),
                    AccountMeta::new_readonly(*wallet_signer_info.key, true),
                    AccountMeta::new(*from_info.key, false),
                    AccountMeta::new(*mint_info.key, false),
                    AccountMeta::new_readonly(*from_auth_info.key, true),
                ],
            ),
            &[
                wallet_info.clone(),
                wallet_pda_info.clone(),
                spl_token_program_info.clone(),
                wallet_signer_info.clone(),
                wallet_program_info.clone(),
                from_info.clone(),
                mint_info.clone(),
                from_auth_info.clone(),
            ],
            &[&seeds],
        )?;

        Ok(())
    }

    pub fn process_teleport_out(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        txhash: &[u8; 32],
        amount: u64,
        decimals: u8,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let config_info = next_account_info(account_info_iter)?;
        let admin_info = next_account_info(account_info_iter)?;
        let admin_auth_info = next_account_info(account_info_iter)?;

        // check config
        let config = Self::get_config(program_id, config_info)?;
        if config.is_frozen {
            return Err(TeleportError::Freeze.into());
        }

        // check admin & auth
        if !config.contain_admin(admin_info.key) {
            msg!("config doesn't contain admin key");
            return Err(TeleportError::UnexpectedError.into());
        }
        let mut admin = state::Admin::try_from_slice(&admin_info.data.borrow())?;
        if !admin.is_init {
            return Err(TeleportError::UninitializedAccount.into());
        }

        if admin_auth_info.key != &admin.auth {
            msg!("admin auth mismatch");
            return Err(TeleportError::UnexpectedError.into());
        }
        if !admin_auth_info.is_signer {
            return Err(TeleportError::MissingRequiredSignature.into());
        }

        if admin.allowance < amount {
            msg!("admin allowance isn't enough");
            return Err(TeleportError::UnexpectedError.into());
        }
        admin.allowance -= amount;
        admin.serialize(&mut *admin_info.data.borrow_mut())?;

        Self::teleport_out(program_id, account_info_iter, txhash, amount, decimals)
    }

    fn teleport_out(
        program_id: &Pubkey,
        account_info_iter: &mut std::slice::Iter<solana_program::account_info::AccountInfo>,
        txhash: &[u8; 32],
        amount: u64,
        decimals: u8,
    ) -> ProgramResult {
        let record_info = next_account_info(account_info_iter)?;
        let wallet_info = next_account_info(account_info_iter)?;
        let wallet_signer_info = next_account_info(account_info_iter)?;
        let fee_payer_info = next_account_info(account_info_iter)?;
        let wallet_program_info = next_account_info(account_info_iter)?;
        let mint_info = next_account_info(account_info_iter)?;
        let to_info = next_account_info(account_info_iter)?;
        let mint_auth_info = next_account_info(account_info_iter)?;
        let spl_token_program_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let teleport_program_info = next_account_info(account_info_iter)?;
        let rent_sysvar_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_sysvar_info)?;
        // check wallet program
        let expected_multisig_program = Pubkey::from_str(state::MULTISIG_PROGRAM_KEY).unwrap();
        if wallet_program_info.key != &expected_multisig_program {
            msg!("unexpected multisig program");
            return Err(TeleportError::UnexpectedError.into());
        }

        // check token program
        if spl_token_program_info.key != &spl_token::id() {
            msg!("unexpected token program");
            return Err(TeleportError::UnexpectedError.into());
        }

        // check system program
        if system_program_info.key != &system_program::id() {
            msg!("unexpected system program id");
            return Err(TeleportError::UnexpectedError.into());
        }

        // check teleport program
        if teleport_program_info.key != program_id {
            msg!("unexpected teleport program");
            return Err(TeleportError::UnexpectedError.into());
        }

        let (pda, bump) = Pubkey::find_program_address(&[&txhash[..]], &program_id);
        if record_info.key != &pda {
            msg!("record account mismatch");
            return Err(TeleportError::UnexpectedError.into());
        }
        // TODO assign owner for account hold lamports
        if record_info.try_lamports().unwrap() != 0 {
            msg!("record lamports is not zero");
            return Err(TeleportError::UnexpectedError.into());
        }
        if !record_info.data_is_empty() {
            msg!("record data is not empty");
            return Err(TeleportError::UnexpectedError.into());
        }

        // create teleport out account
        let seeds: &[&[_]] = &[&txhash[..], &[bump]];
        invoke_signed(
            &system_instruction::create_account(
                &fee_payer_info.key,
                &record_info.key,
                rent.minimum_balance(state::TeleportOutRecord::LEN),
                state::TeleportOutRecord::LEN as u64,
                &program_id,
            ),
            &[fee_payer_info.clone(), record_info.clone()],
            &[&seeds],
        )?;

        // init teleport out account
        invoke(
            &crate::instruction::init_teleport_out_record(&program_id, &record_info.key).unwrap(),
            &[record_info.clone()],
        )?;

        // mint blt token
        let seeds: &[&[_]] = &[
            state::SIGNER_SEED,
            &[Pubkey::find_program_address(&[state::SIGNER_SEED], &program_id).1],
        ];

        let mut data = vec![
            3, // u8, wallet program invoke instruction
            3, // u8, invoke program idx
            3, 0, // u16, total account, little endian
            5, 1, // u8, u8, account idx, not signer writable
            6, 1, // ..
            7, 2,  // readonly singer
            14, // u8, burn instruction in token program
        ];
        data.extend(amount.to_le_bytes().iter().cloned());
        data.push(decimals);

        invoke_signed(
            &Instruction::new_with_bytes(
                *wallet_program_info.key,
                &data[..],
                vec![
                    AccountMeta::new(*wallet_info.key, false),
                    AccountMeta::new_readonly(*mint_auth_info.key, false),
                    AccountMeta::new_readonly(*fee_payer_info.key, false),
                    AccountMeta::new_readonly(*spl_token_program_info.key, false),
                    AccountMeta::new_readonly(*wallet_signer_info.key, true),
                    AccountMeta::new(*mint_info.key, false),
                    AccountMeta::new(*to_info.key, false),
                    AccountMeta::new_readonly(*mint_auth_info.key, false),
                ],
            ),
            &[
                wallet_info.clone(),
                mint_auth_info.clone(),
                fee_payer_info.clone(),
                spl_token_program_info.clone(),
                wallet_signer_info.clone(),
                wallet_program_info.clone(),
                mint_info.clone(),
                to_info.clone(),
            ],
            &[&seeds],
        )?;

        Ok(())
    }

    pub fn process_deposit_allowance(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        allowance: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let owner_info = next_account_info(account_info_iter)?;
        let admin_info = next_account_info(account_info_iter)?;

        Self::only_owner(owner_info)?;

        let mut admin = state::Admin::try_from_slice(&admin_info.data.borrow())?;
        if !admin.is_init {
            return Err(TeleportError::UninitializedAccount.into());
        }

        admin.allowance = admin.allowance.checked_add(allowance).unwrap();

        admin
            .serialize(&mut *admin_info.data.borrow_mut())
            .map_err(|e| e.into())
    }

    pub fn process_close_teleport_out_record(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let config_info = next_account_info(account_info_iter)?;
        let admin_info = next_account_info(account_info_iter)?;
        let admin_auth_info = next_account_info(account_info_iter)?;
        let teleport_out_record_info = next_account_info(account_info_iter)?;
        let target_info = next_account_info(account_info_iter)?;

        // check config
        let config = Self::get_config(program_id, config_info)?;
        if config.is_frozen {
            return Err(TeleportError::Freeze.into());
        }

        // check admin & auth
        if !config.contain_admin(admin_info.key) {
            msg!("config doesn't contain admin key");
            return Err(TeleportError::UnexpectedError.into());
        }
        let admin = state::Admin::try_from_slice(&admin_info.data.borrow())?;
        if !admin.is_init {
            return Err(TeleportError::UninitializedAccount.into());
        }

        if admin_auth_info.key != &admin.auth {
            msg!("admin auth mismatch");
            return Err(TeleportError::UnexpectedError.into());
        }
        if !admin_auth_info.is_signer {
            return Err(TeleportError::MissingRequiredSignature.into());
        }

        // check teleport_out_record_info
        Self::get_teleport_out_record(program_id, teleport_out_record_info)?;

        let dest_starting_lamports = target_info.lamports();
        **target_info.lamports.borrow_mut() = dest_starting_lamports
            .checked_add(teleport_out_record_info.lamports())
            .ok_or(ProgramError::InvalidAccountData)?;
        **teleport_out_record_info.lamports.borrow_mut() = 0;

        Ok(())
    }

    fn only_owner(account_info: &AccountInfo) -> ProgramResult {
        let owner = Pubkey::from_str(state::OWNER_KEY).unwrap();
        if account_info.key != &owner {
            msg!("owner mismatch");
            return Err(TeleportError::AuthFailed.into());
        }

        if !account_info.is_signer {
            msg!("owner should be a singer");
            return Err(TeleportError::AuthFailed.into());
        }

        Ok(())
    }

    fn get_config(
        program_id: &Pubkey,
        config_info: &AccountInfo,
    ) -> Result<state::Config, ProgramError> {
        if config_info.owner != program_id {
            return Err(TeleportError::IncorrectProgramAccount.into());
        }

        if config_info.data_len() != state::Config::LEN {
            return Err(TeleportError::IncorrectProgramAccount.into());
        }

        let config = state::Config::try_from_slice(&config_info.data.borrow())?;
        if !config.is_init {
            return Err(TeleportError::UninitializedAccount.into());
        }

        return Ok(config);
    }

    fn get_teleport_out_record(
        program_id: &Pubkey,
        teleport_out_record_info: &AccountInfo,
    ) -> Result<state::TeleportOutRecord, ProgramError> {
        if teleport_out_record_info.owner != program_id {
            return Err(TeleportError::IncorrectProgramAccount.into());
        }

        if teleport_out_record_info.data_len() != state::TeleportOutRecord::LEN {
            return Err(TeleportError::IncorrectProgramAccount.into());
        }

        let teleport_out_record =
            state::TeleportOutRecord::try_from_slice(&teleport_out_record_info.data.borrow())?;
        if !teleport_out_record.is_init {
            return Err(TeleportError::UninitializedAccount.into());
        }

        return Ok(teleport_out_record);
    }
}
