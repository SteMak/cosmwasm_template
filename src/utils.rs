// The file is responsible for storing all custom structs

use cosmwasm_std::{Addr, CanonicalAddr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Birthday is type for calculating user age
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, JsonSchema)]
pub struct Birthday {
  // Day is optional, may be in [1:366]
  pub day: Option<u16>,

  // Year may be [1756:current year]
  pub year: u16,
}

// Defining such types is important as it keeps code declarative
pub type CityName = String;
pub type Nickname = String;
pub type Email = String;

// Config is struct that is stored in cold storage by specific key
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
  // Important: all addresses should be stored in CanonicalAddr type
  // Contract owner address
  pub maintainer: CanonicalAddr,

  // Counter of registered cities
  pub cities_count: u64,
}

// City is struct that is stored in mapping by u64 key
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, JsonSchema)]
pub struct City {
  // City metagata
  pub city_name: CityName,
  pub power_level: u8,

  // Counter of city members
  pub members_count: u64,
}

// CityResponse is struct for representing City when querying
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, JsonSchema)]
pub struct CityResponse {
  // City id
  pub id: u64,

  // City metagata
  pub name: CityName,
  pub power_level: u8,

  // Number of city members
  pub population: u64,
}

// Person is struct that is stored in mapping by account address key
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, JsonSchema)]
pub struct Person {
  // Person metagata
  pub birthday: Birthday,
  pub nickname: Nickname,
  // Person may not provide email to stay anonymus
  pub email: Option<Email>,

  // Counter of cities where person is registered
  pub cities_count: u64,
}

// PersonResponse is struct for representing Person when querying
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, JsonSchema)]
pub struct PersonResponse {
  // Person creator address
  pub address: Addr,

  // Person metagata
  pub birthday: Birthday,
  pub nickname: Nickname,
  pub email: Option<Email>,

  // Number of cities where person is registered
  pub resident_times: u64,
}
