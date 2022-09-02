// The file is responsible for storing list of custom error types

use cosmwasm_std::{StdError, StdResult};
use thiserror::Error;

use crate::utils::{CityName, Nickname};

// Helper function for wrapping StdError::NotFound to ContractError::NotFound
// <T> is generic for any Ok(result) data type

pub fn wrap_not_found<T>(result: StdResult<T>) -> Result<T, ContractError> {
  match result {
    Ok(data) => Ok(data),
    Err(err) => match err {
      StdError::NotFound { kind } => Err(ContractError::NotFound {
        kind: kind.split("::").last().unwrap_or(&kind).to_string(),
      }),
      _ => Err(ContractError::Std(err)),
    },
  }
}

// ContractError enum stores error types with text representation
#[derive(Error, Debug)]
pub enum ContractError {
  // Std() is used for wrapping StdError to ContractError
  #[error("{0}")]
  Std(#[from] StdError),

  #[error("Inconsistent input data provided")]
  InconsistentData {},

  #[error("You are not maintainer")]
  Unauthorized {},

  #[error("You are already maintainer")]
  AlreadyMaintainer {},

  // Use { field } if you are sure that the field is not empty, results in 'error start field_value end'
  #[error("You don't satisfy maintainer requirements ({requirement})")]
  InconsistentMaintainer { requirement: String },

  #[error("{kind} with this identifier is not found")]
  NotFound { kind: String },

  // Use { field:? } if field could be empty, results in 'error start "field_value" end'
  #[error("Person with the address already exists (nickname: {nickname:?})")]
  PersonAlreadyRegistered { nickname: Nickname },

  #[error("Person is already registered in the city (nickname: {nickname:?}, city name: {city_name:?})")]
  PersonAlreadyRegisteredInCity { nickname: Nickname, city_name: CityName },
}
