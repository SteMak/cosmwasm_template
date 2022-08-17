// The file contains storage structs and helper functions for interactions with stored data

// Import section contains all needed imports
use cosmwasm_std::{CanonicalAddr, Storage};
use cw_storage_plus::{Item, Map};

// The crate imports are responsible for import from anothe project file
use crate::{
  error::ContractError,
  utils::{Birthday, City, Config, Person},
};

// Config instance that is stored by specific key
const CONFIG_INSTANCE: Item<Config> = Item::new("config_key");

// Mapping instances that are stored by specific keys
// CanonicalAddr could be represented as &[u8] which is valid value for mpping key
// PersonByAddress is mapping from person address to Person object
const PERSON_BY_ADDRESS: Map<&[u8], Person> = Map::new("person_by_address");
// CityByID is mapping from city index to City object
const CITY_BY_ID: Map<u64, City> = Map::new("city_by_id");

// Mapping instances responsible for link between person and city that are stored by specific keys
// PersonAddressByCityIDAndPersonInCityID is mapping from city index and person in the city index to person address
const PERSON_ADDRESS_BY_CITY_ID_AND_PERSON_IN_CITY_ID: Map<(u64, u64), CanonicalAddr> =
  Map::new("person_address_by_city_id_and_person_in_city_id");
// CityIDByPersonAddressAndCityInPersonID is mapping from person address and city in the person index to city index
const CITY_ID_BY_PERSON_ADDRESS_AND_CITY_IN_PERSON_ID: Map<(&[u8], u64), u64> =
  Map::new("city_id_by_person_address_and_city_in_person_id");
// PersonInCityIDAndCityInPersonIDByPersonAddressAndCityID is mapping from person address and city index
// to person in the city index and city in the person index
const PERSON_IN_CITY_ID_AND_CITY_IN_PERSON_ID_BY_PERSON_ADDRESS_AND_CITY_ID: Map<(&[u8], u64), (u64, u64)> =
  Map::new("person_in_city_id_and_city_in_person_id_by_person_address_and_city_id");

// Helper functions for loading person/city data
pub fn get_person(store: &dyn Storage, addr: CanonicalAddr) -> Result<Person, ContractError> {
  Ok(PERSON_BY_ADDRESS.load(store, &addr)?)
}
pub fn get_city(store: &dyn Storage, city_id: u64) -> Result<City, ContractError> {
  Ok(CITY_BY_ID.load(store, city_id)?)
}

// Helper functions for accessing person/city from city/person data
pub fn get_city_id_by_person(store: &dyn Storage, addr: CanonicalAddr, index: u64) -> Result<u64, ContractError> {
  Ok(CITY_ID_BY_PERSON_ADDRESS_AND_CITY_IN_PERSON_ID.load(store, (&addr, index))?)
}
pub fn get_person_address_by_city(
  store: &dyn Storage,
  city_id: u64,
  index: u64,
) -> Result<CanonicalAddr, ContractError> {
  Ok(PERSON_ADDRESS_BY_CITY_ID_AND_PERSON_IN_CITY_ID.load(store, (city_id, index))?)
}

// Helper functions for loading/saving config data
pub fn get_storage(store: &dyn Storage) -> Result<Config, ContractError> {
  Ok(CONFIG_INSTANCE.load(store)?)
}
pub fn set_storage(store: &mut dyn Storage, config: &Config) -> Result<(), ContractError> {
  CONFIG_INSTANCE.save(store, config)?;

  Ok(())
}

// Helper functions for updating counters
fn increment_city_counter(store: &mut dyn Storage) -> Result<(), ContractError> {
  let mut config = get_storage(store)?;
  config.cities_count += 1;
  set_storage(store, &config)?;

  Ok(())
}
fn crement_person_and_city_counters(
  store: &mut dyn Storage,
  addr: CanonicalAddr,
  city_id: u64,
  increment: bool,
) -> Result<(), ContractError> {
  let mut person = get_person(store, addr.clone())?;
  let mut city = get_city(store, city_id)?;

  if increment {
    person.cities_count += 1;
    city.members_count += 1;
  } else {
    person.cities_count -= 1;
    city.members_count -= 1;
  }

  PERSON_BY_ADDRESS.save(store, &addr, &person)?;
  CITY_BY_ID.save(store, city_id, &city)?;

  Ok(())
}

// Helper function for adding new city
pub fn create_city(store: &mut dyn Storage, city_name: String, power_level: u8) -> Result<(), ContractError> {
  let config = get_storage(store)?.cities_count;
  CITY_BY_ID.save(
    store,
    config,
    &City {
      city_name: city_name,
      power_level: power_level,
      members_count: 0,
    },
  )?;

  increment_city_counter(store)?;

  Ok(())
}

// Helper function for adding new person
pub fn create_person(
  store: &mut dyn Storage,
  addr: CanonicalAddr,
  birthday: Birthday,
  nickname: String,
  email: Option<String>,
) -> Result<(), ContractError> {
  if let Some(person) = PERSON_BY_ADDRESS.may_load(store, &addr)? {
    return Err(ContractError::PersonAlreadyRegistered {
      nickname: person.nickname,
    });
  };

  PERSON_BY_ADDRESS.save(
    store,
    &addr,
    &Person {
      birthday: birthday,
      nickname: nickname,
      email: email,
      cities_count: 0,
    },
  )?;

  Ok(())
}

// Helper function for updating person metadata
pub fn update_person(
  store: &mut dyn Storage,
  addr: CanonicalAddr,
  nickname: String,
  email: Option<String>,
) -> Result<(), ContractError> {
  let mut person = get_person(store, addr.clone())?;

  person.nickname = nickname;
  person.email = email;

  PERSON_BY_ADDRESS.save(store, &addr, &person)?;

  Ok(())
}

// Helper function for registring person in city
pub fn register_in_city(store: &mut dyn Storage, addr: CanonicalAddr, city_id: u64) -> Result<(), ContractError> {
  let person = get_person(store, addr.clone())?;
  let city = get_city(store, city_id)?;

  if let Some((_, _)) =
    PERSON_IN_CITY_ID_AND_CITY_IN_PERSON_ID_BY_PERSON_ADDRESS_AND_CITY_ID.may_load(store, (&addr, city_id))?
  {
    return Err(ContractError::PersonAlreadyRegisteredInCity {
      nickname: person.nickname,
      city_name: city.city_name,
    });
  };

  // Adding new person in city
  PERSON_ADDRESS_BY_CITY_ID_AND_PERSON_IN_CITY_ID.save(store, (city_id, city.members_count), &addr)?;

  // Adding new city where person live
  CITY_ID_BY_PERSON_ADDRESS_AND_CITY_IN_PERSON_ID.save(store, (&addr, person.cities_count), &city_id)?;

  // Adding link beween user and city
  PERSON_IN_CITY_ID_AND_CITY_IN_PERSON_ID_BY_PERSON_ADDRESS_AND_CITY_ID.save(
    store,
    (&addr, city_id),
    &(city.members_count, person.cities_count),
  )?;

  // Incrementing counters
  crement_person_and_city_counters(store, addr, city_id, true)?;

  Ok(())
}

// Helper function for registring person in city
pub fn unregister_from_city(store: &mut dyn Storage, addr: CanonicalAddr, city_id: u64) -> Result<(), ContractError> {
  let person = get_person(store, addr.clone())?;
  let city = get_city(store, city_id)?;

  let (person_in_city, city_in_person) =
    PERSON_IN_CITY_ID_AND_CITY_IN_PERSON_ID_BY_PERSON_ADDRESS_AND_CITY_ID.load(store, (&addr, city_id))?;

  // Replace current person_in_city and city_in_person with the last ones
  if person_in_city < city.members_count - 1 {
    let last_pic_address =
      PERSON_ADDRESS_BY_CITY_ID_AND_PERSON_IN_CITY_ID.load(store, (city_id, city.members_count - 1))?;
    PERSON_ADDRESS_BY_CITY_ID_AND_PERSON_IN_CITY_ID.save(store, (city_id, person_in_city), &last_pic_address)?;

    let (_, city_id_of_moved_user) = PERSON_IN_CITY_ID_AND_CITY_IN_PERSON_ID_BY_PERSON_ADDRESS_AND_CITY_ID
      .load(store, (&last_pic_address, city_id))?;
    PERSON_IN_CITY_ID_AND_CITY_IN_PERSON_ID_BY_PERSON_ADDRESS_AND_CITY_ID.save(
      store,
      (&last_pic_address, city_id),
      &(person_in_city, city_id_of_moved_user),
    )?;
  }
  if city_in_person < person.cities_count - 1 {
    let last_cip_id = CITY_ID_BY_PERSON_ADDRESS_AND_CITY_IN_PERSON_ID.load(store, (&addr, person.cities_count - 1))?;
    CITY_ID_BY_PERSON_ADDRESS_AND_CITY_IN_PERSON_ID.save(store, (&addr, city_in_person), &last_cip_id)?;

    let (person_id_of_moved_city, _) =
      PERSON_IN_CITY_ID_AND_CITY_IN_PERSON_ID_BY_PERSON_ADDRESS_AND_CITY_ID.load(store, (&addr, last_cip_id))?;
    PERSON_IN_CITY_ID_AND_CITY_IN_PERSON_ID_BY_PERSON_ADDRESS_AND_CITY_ID.save(
      store,
      (&addr, last_cip_id),
      &(person_id_of_moved_city, city_in_person),
    )?;
  }

  // Removing link beween user and city
  PERSON_IN_CITY_ID_AND_CITY_IN_PERSON_ID_BY_PERSON_ADDRESS_AND_CITY_ID.remove(store, (&addr, city_id));

  // Removing copy of moved data
  PERSON_ADDRESS_BY_CITY_ID_AND_PERSON_IN_CITY_ID.remove(store, (city_id, city.members_count - 1));
  CITY_ID_BY_PERSON_ADDRESS_AND_CITY_IN_PERSON_ID.remove(store, (&addr, person.cities_count - 1));

  // Decrementing counters
  crement_person_and_city_counters(store, addr, city_id, false)?;

  Ok(())
}
