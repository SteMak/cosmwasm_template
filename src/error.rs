use cosmwasm_std::StdError;
use thiserror::Error;

use crate::utils::{CityName, Nickname};

#[derive(Error, Debug)]
pub enum ContractError {
  #[error("{0}")]
  Std(#[from] StdError),

  #[error("Inconsistent input data provided")]
  InconsistentData {},

  #[error("You are not maintainer")]
  Unauthorized {},

  #[error("You are already maintainer")]
  AlreadyMaintainer {},

  #[error("You don't satisfy maintainer requirements ({requirement:?})")]
  InconsistentMaintainer { requirement: String },

  #[error("Person with the address already exists (nickname: {nickname:?})")]
  PersonAlreadyRegistered { nickname: Nickname },

  #[error("Person is already registered in the city (nickname: {nickname:?}, city name: {city_name:?})")]
  PersonAlreadyRegisteredInCity { nickname: Nickname, city_name: CityName },
}
