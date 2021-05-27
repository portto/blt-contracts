//! Error types

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use solana_program::{
    decode_error::DecodeError, msg, program_error::PrintProgramError, program_error::ProgramError,
};
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum TeleportError {
    #[error("AuthFailed")]
    AuthFailed,
    #[error("AlreadyInUse")]
    AlreadyInUse,
    #[error("NotRentExempt")]
    NotRentExempt,
}

impl From<TeleportError> for ProgramError {
    fn from(e: TeleportError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for TeleportError {
    fn type_of() -> &'static str {
        "Teleport Error"
    }
}

impl PrintProgramError for TeleportError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            TeleportError::AuthFailed => msg!("Auth Failed"),
            TeleportError::AlreadyInUse => msg!("Already In Use"),
            TeleportError::NotRentExempt => msg!("Not Rent Exempt"),
        }
    }
}
