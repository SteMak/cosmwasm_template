use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::utils::{Birthday, CityName, CityResponse, Email, Nickname, PersonResponse};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
  RegisterCity {
    name: CityName,
    power_level: u8,
  },
  RegisterPerson {
    birthday: Birthday,
    nickname: Nickname,
    email: Option<Email>,
  },
  UpdatePerson {
    nickname: Nickname,
    email: Option<Email>,
  },

  RegisterInCity {
    city_id: u64,
  },
  UnregisterFromCity {
    city_id: u64,
  },

  BecomeMaintainer {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
  LookMaintainer {},

  LookPerson { person: Addr },
  LookCities { start_id: u64, limit: u64 },

  LookPersonCities { person: Addr, start_id: u64, limit: u64 },
  LookCityPeople { city: u64, start_id: u64, limit: u64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResponseMsg {
  LookMaintainer { maintainer: Addr },

  LookPerson { person: PersonResponse },
  LookCities { cities: Vec<CityResponse> },

  LookPersonCities { cities: Vec<CityResponse> },
  LookCityPeople { people: Vec<PersonResponse> },
}
