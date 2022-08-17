// The file is responsible for storing all custom structs

use cosmwasm_std::{Addr, CanonicalAddr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, JsonSchema)]
pub struct Birthday {
  pub day: Option<u16>,
  pub year: u16,
}

pub type CityName = String;
pub type Nickname = String;
pub type Email = String;

// Derive implements features for the struct:
// - Serialize - adds possibility to code the struct into JSON (is needed when other contracts try to get the storage)
// - Deserialize - adds possibility to decode the struct from JSON
// - Clone - adds possibility to create duplicate of the struct by calling struct_instance.clone()
// - Debug - adds possibility to use the structure in asserts
// - PartialEq - adds possibility to compare instances of the struct
// - JsonSchema - adds possibility to create JSON schema of the struct
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// Config is struct that is stored in cold storage by specific key
pub struct Config {
  // Important: all addresses should be stored in CanonicalAddr type
  // Contract owner address
  pub maintainer: CanonicalAddr,

  // Counter of registered cities
  pub cities_count: u64,
}

// Derives description is upper
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, JsonSchema)]
// City is struct that is stored in mapping by u64 key
pub struct City {
  // City metagata
  pub city_name: CityName,
  pub power_level: u8,

  // Counter of city members
  pub members_count: u64,
}

// Derives description is upper
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, JsonSchema)]
pub struct CityResponse {
  pub id: u64,

  pub name: CityName,
  pub power_level: u8,
  pub population: u64,
}

// Derives description is upper
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, JsonSchema)]
// Person is struct that is stored in mapping by account address key
pub struct Person {
  // Person metagata
  pub birthday: Birthday,
  pub nickname: Nickname,
  // Person may not provide email to stay anonymus
  pub email: Option<Email>,

  // Counter of cities where person is registered
  pub cities_count: u64,
}

// Derives description is upper
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, JsonSchema)]
pub struct PersonResponse {
  pub address: Addr,

  pub birthday: Birthday,
  pub nickname: Nickname,
  pub email: Option<Email>,

  pub resident_times: u64,
}
