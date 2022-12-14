// The file is responsible for storing list of contract methods

use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ResponseMsg};
use crate::state::{
  create_city, create_person, get_city, get_city_id_by_person, get_person, get_person_address_by_city, get_storage,
  register_in_city, set_storage, unregister_from_city, update_person,
};
use crate::utils::{Birthday, CityName, CityResponse, Config, Email, Nickname, PersonResponse};

const YEAR_IN_SECONDS: u64 = 31556952;
const DAY_IN_SECONDS: u64 = 86400;

const MAINTAINER_REQUIREMENT_NAME: &str = "Super_Maintainer_887";
const MAINTAINER_REQUIREMENT_AGE: u8 = 17;

// Instantiate contract entrypoint
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, _: Env, info: MessageInfo, _: InstantiateMsg) -> Result<Response, ContractError> {
  // Initialize config structure
  let config = Config {
    maintainer: deps.api.addr_canonicalize(info.sender.as_str())?,
    cities_count: 0,
  };

  // Store config to cold storage
  set_storage(deps.storage, &config)?;

  // Return default Ok response
  Ok(Response::default())
}

// Execute contract entrypoint
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, ContractError> {
  // Route call to corresponding method
  match msg {
    ExecuteMsg::BecomeMaintainer {} => execute_become_maintainer(deps, env, info),

    ExecuteMsg::RegisterCity { name, power_level } => execute_register_city(deps, env, info, name, power_level),
    ExecuteMsg::RegisterPerson {
      birthday,
      nickname,
      email,
    } => execute_register_person(deps, env, info, birthday, nickname, email),
    ExecuteMsg::UpdatePerson { nickname, email } => execute_update_person(deps, env, info, nickname, email),

    ExecuteMsg::RegisterInCity { city_id } => execute_register_in_city(deps, env, info, city_id),
    ExecuteMsg::UnregisterFromCity { city_id } => execute_unregister_from_city(deps, env, info, city_id),
  }
}

fn execute_become_maintainer(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
  // Get config from storage
  let mut config = get_storage(deps.storage)?;

  // Check current maintainer is not caller
  let canonical_sender = deps.api.addr_canonicalize(info.sender.as_str())?;
  if canonical_sender == config.maintainer {
    return Err(ContractError::AlreadyMaintainer {});
  }

  // Get caller person
  let person = get_person(deps.storage, canonical_sender.clone())?;

  // Check caller name
  if person.nickname != MAINTAINER_REQUIREMENT_NAME {
    return Err(ContractError::InconsistentMaintainer {
      requirement: "You are not crazy enough".to_string(),
    });
  }

  // Check caller age
  if (1970 * YEAR_IN_SECONDS + env.block.time.seconds())
    < MAINTAINER_REQUIREMENT_AGE as u64 * YEAR_IN_SECONDS
      + person.birthday.year as u64 * YEAR_IN_SECONDS
      + person.birthday.day.unwrap_or(366) as u64 * DAY_IN_SECONDS
  {
    return Err(ContractError::InconsistentMaintainer {
      requirement: "You are too young".to_string(),
    });
  }

  // Update config
  config.maintainer = canonical_sender;

  // Store config to cold storage
  set_storage(deps.storage, &config)?;

  // Return default Ok response
  Ok(Response::default())
}

fn execute_register_city(
  deps: DepsMut,
  _: Env,
  info: MessageInfo,
  name: CityName,
  power_level: u8,
) -> Result<Response, ContractError> {
  // Get config from storage
  let config = get_storage(deps.storage)?;

  // Check caller is maintainer
  let canonical_sender = deps.api.addr_canonicalize(info.sender.as_str())?;
  if canonical_sender != config.maintainer {
    return Err(ContractError::Unauthorized {});
  }

  // Register new city
  create_city(deps.storage, name, power_level)?;

  // Return default Ok response
  Ok(Response::default())
}

fn execute_register_person(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  birthday: Birthday,
  nickname: Nickname,
  email: Option<Email>,
) -> Result<Response, ContractError> {
  // Validate birthday day
  if let Some(day) = birthday.day {
    if day > 366 || day == 0 {
      return Err(ContractError::InconsistentData {});
    }
  }

  // Validate birthday year
  if birthday.year < 1756 || birthday.year > (env.block.time.seconds() / YEAR_IN_SECONDS) as u16 + 1970 {
    return Err(ContractError::InconsistentData {});
  }

  // Check person is not registered yet
  let canonical_sender = deps.api.addr_canonicalize(info.sender.as_str())?;
  create_person(deps.storage, canonical_sender, birthday, nickname, email)?;

  // Return default Ok response
  Ok(Response::default())
}

fn execute_update_person(
  deps: DepsMut,
  _: Env,
  info: MessageInfo,
  nickname: Nickname,
  email: Option<Email>,
) -> Result<Response, ContractError> {
  // Update person by address
  let canonical_sender = deps.api.addr_canonicalize(info.sender.as_str())?;
  update_person(deps.storage, canonical_sender, nickname, email)?;

  // Return default Ok response
  Ok(Response::default())
}

fn execute_register_in_city(deps: DepsMut, _: Env, info: MessageInfo, city_id: u64) -> Result<Response, ContractError> {
  // Create link betwee user and city
  let canonical_sender = deps.api.addr_canonicalize(info.sender.as_str())?;
  register_in_city(deps.storage, canonical_sender, city_id)?;

  // Return default Ok response
  Ok(Response::default())
}

fn execute_unregister_from_city(
  deps: DepsMut,
  _: Env,
  info: MessageInfo,
  city_id: u64,
) -> Result<Response, ContractError> {
  // Remove link betwee user and city
  let canonical_sender = deps.api.addr_canonicalize(info.sender.as_str())?;
  unregister_from_city(deps.storage, canonical_sender, city_id)?;

  // Return default Ok response
  Ok(Response::default())
}

// Query contract entrypoint
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
  // Route call to corresponding method
  match msg {
    QueryMsg::LookMaintainer {} => Ok(to_binary(&query_look_maintainer(deps)?)?),
    QueryMsg::LookPerson { person } => Ok(to_binary(&query_look_person(deps, person)?)?),
    QueryMsg::LookCities { start_id, limit } => Ok(to_binary(&query_look_cities(deps, start_id, limit)?)?),
    QueryMsg::LookPersonCities {
      person,
      start_id,
      limit,
    } => Ok(to_binary(&query_look_person_cities(deps, person, start_id, limit)?)?),
    QueryMsg::LookCityPeople { city, start_id, limit } => {
      Ok(to_binary(&query_look_city_people(deps, city, start_id, limit)?)?)
    }
  }
}

fn query_look_maintainer(deps: Deps) -> Result<ResponseMsg, ContractError> {
  // Get config from storage
  let config = get_storage(deps.storage)?;

  // Return corresponding responce
  Ok(ResponseMsg::LookMaintainer {
    maintainer: deps.api.addr_humanize(&config.maintainer)?,
  })
}

fn query_look_person(deps: Deps, addr: Addr) -> Result<ResponseMsg, ContractError> {
  // Get person by address
  let canonical_sender = deps.api.addr_canonicalize(addr.as_str())?;
  let person = get_person(deps.storage, canonical_sender.clone())?;

  // Return corresponding responce
  Ok(ResponseMsg::LookPerson {
    person: PersonResponse {
      address: addr,
      birthday: person.birthday,
      nickname: person.nickname,
      email: person.email,
      resident_times: person.cities_count,
    },
  })
}

fn query_look_cities(deps: Deps, start_id: u64, limit: u64) -> Result<ResponseMsg, ContractError> {
  // Get config from storage
  let config = get_storage(deps.storage)?;

  // Init result
  let mut result: Vec<CityResponse> = vec![];

  // Loop through ids
  for i in start_id..(start_id + limit) {
    // Validate id
    if config.cities_count > i {
      // Get city by id
      let city = get_city(deps.storage, i)?;
      // Update result
      result.push(CityResponse {
        id: i,
        name: city.city_name,
        power_level: city.power_level,
        population: city.members_count,
      })
    } else {
      break;
    }
  }

  // Return corresponding responce
  Ok(ResponseMsg::LookCities { cities: result })
}

fn query_look_person_cities(deps: Deps, addr: Addr, start_id: u64, limit: u64) -> Result<ResponseMsg, ContractError> {
  // Get person by address
  let canonical_sender = deps.api.addr_canonicalize(addr.as_str())?;
  let person = get_person(deps.storage, canonical_sender.clone())?;

  // Init result
  let mut result: Vec<CityResponse> = vec![];

  // Loop through ids
  for i in start_id..(start_id + limit) {
    // Validate id
    if person.cities_count > i {
      // Get city id
      let city_id = get_city_id_by_person(deps.storage, canonical_sender.clone(), i)?;
      // Get city by id
      let city = get_city(deps.storage, city_id)?;
      // Update result
      result.push(CityResponse {
        id: city_id,
        name: city.city_name,
        power_level: city.power_level,
        population: city.members_count,
      })
    } else {
      break;
    }
  }

  // Return corresponding responce
  Ok(ResponseMsg::LookPersonCities { cities: result })
}

fn query_look_city_people(deps: Deps, city_id: u64, start_id: u64, limit: u64) -> Result<ResponseMsg, ContractError> {
  // Get person by id
  let city = get_city(deps.storage, city_id)?;

  // Init result
  let mut result: Vec<PersonResponse> = vec![];

  // Loop through ids
  for i in start_id..(start_id + limit) {
    // Validate id
    if city.members_count > i {
      // Get person address
      let addr = get_person_address_by_city(deps.storage, city_id, i)?;
      // Get person by address
      let person = get_person(deps.storage, addr.clone())?;
      // Update result
      result.push(PersonResponse {
        address: deps.api.addr_humanize(&addr)?,
        birthday: person.birthday,
        nickname: person.nickname,
        email: person.email,
        resident_times: person.cities_count,
      })
    } else {
      break;
    }
  }

  // Return corresponding responce
  Ok(ResponseMsg::LookCityPeople { people: result })
}

#[cfg(test)]
mod tests {

  use crate::error::wrap_not_found;

  use super::*;
  use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
  use cosmwasm_std::{from_binary, Addr, DivideByZeroError};
  use cosmwasm_std::{StdError, Timestamp};

  #[test]
  fn check_error_wrapper() {
    match wrap_not_found(Ok("data")) {
      Ok(data) => assert_eq!(data, "data"),
      Err(_) => unreachable!(),
    }

    match wrap_not_found(Ok(())) {
      Ok(data) => assert_eq!(data, ()),
      Err(_) => unreachable!(),
    }

    match wrap_not_found::<()>(Err(StdError::NotFound {
      kind: "kind".to_string(),
    })) {
      Ok(_) => unreachable!(),
      Err(err) => match err {
        ContractError::NotFound { kind } => assert_eq!(kind, "kind".to_string()),
        _ => unreachable!(),
      },
    }

    match wrap_not_found::<()>(Err(StdError::DivideByZero {
      source: DivideByZeroError {
        operand: "ohh".to_string(),
      },
    })) {
      Ok(_) => unreachable!(),
      Err(err) => match err {
        ContractError::Std(stderr) => match stderr {
          StdError::DivideByZero { source } => assert_eq!(source.operand, "ohh".to_string()),
          _ => unreachable!(),
        },
        _ => unreachable!(),
      },
    }
  }

  #[test]
  fn initialization() {
    // Get deps for calls
    let mut deps = mock_dependencies();

    // Setup env and info
    let mut env = mock_env();
    env.block.height = 887;
    env.block.time = Timestamp::from_seconds(3);
    let info = mock_info("creator", &[]);

    // Instantiate and check response
    let res = instantiate(deps.as_mut(), env, info, InstantiateMsg {}).unwrap();
    assert_eq!(0, res.messages.len());

    // Query and check result
    let maintainer: ResponseMsg =
      from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::LookMaintainer {}).unwrap()).unwrap();
    assert_eq!(
      maintainer,
      ResponseMsg::LookMaintainer {
        maintainer: Addr::unchecked("creator")
      }
    );
  }

  #[test]
  fn maintainer_takeover() {
    let mut deps = mock_dependencies();

    let mut env = mock_env();
    env.block.height = 887;
    env.block.time = Timestamp::from_seconds(YEAR_IN_SECONDS * 52 + YEAR_IN_SECONDS / 2); // in middle of 2022

    instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), InstantiateMsg {}).unwrap();

    execute(
      deps.as_mut(),
      env.clone(),
      mock_info("user_1", &[]),
      ExecuteMsg::RegisterPerson {
        birthday: Birthday { day: None, year: 2015 },
        nickname: "super_user".to_string(),
        email: None,
      },
    )
    .unwrap();
    execute(
      deps.as_mut(),
      env.clone(),
      mock_info("user_2", &[]),
      ExecuteMsg::RegisterPerson {
        birthday: Birthday { day: None, year: 2000 },
        nickname: "super_user".to_string(),
        email: None,
      },
    )
    .unwrap();
    execute(
      deps.as_mut(),
      env.clone(),
      mock_info("user_3", &[]),
      ExecuteMsg::RegisterPerson {
        birthday: Birthday { day: None, year: 2015 },
        nickname: "Super_Maintainer_887".to_string(),
        email: None,
      },
    )
    .unwrap();
    execute(
      deps.as_mut(),
      env.clone(),
      mock_info("user_4", &[]),
      ExecuteMsg::RegisterPerson {
        birthday: Birthday { day: None, year: 2004 },
        nickname: "Super_Maintainer_887".to_string(),
        email: None,
      },
    )
    .unwrap();

    let bad_takeover = execute(
      deps.as_mut(),
      env.clone(),
      mock_info("creator", &[]),
      ExecuteMsg::BecomeMaintainer {},
    );
    assert!(bad_takeover.is_err());

    let bad_takeover = execute(
      deps.as_mut(),
      env.clone(),
      mock_info("user_1", &[]),
      ExecuteMsg::BecomeMaintainer {},
    );
    assert!(bad_takeover.is_err());

    let bad_takeover = execute(
      deps.as_mut(),
      env.clone(),
      mock_info("user_2", &[]),
      ExecuteMsg::BecomeMaintainer {},
    );
    assert!(bad_takeover.is_err());

    let bad_takeover = execute(
      deps.as_mut(),
      env.clone(),
      mock_info("user_3", &[]),
      ExecuteMsg::BecomeMaintainer {},
    );
    assert!(bad_takeover.is_err());

    execute(
      deps.as_mut(),
      env.clone(),
      mock_info("user_4", &[]),
      ExecuteMsg::BecomeMaintainer {},
    )
    .unwrap();

    let maintainer: ResponseMsg =
      from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::LookMaintainer {}).unwrap()).unwrap();
    assert_eq!(
      maintainer,
      ResponseMsg::LookMaintainer {
        maintainer: Addr::unchecked("user_4")
      }
    );
  }

  #[test]
  fn city_creation() {
    let mut deps = mock_dependencies();

    instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), InstantiateMsg {}).unwrap();

    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("creator", &[]),
      ExecuteMsg::RegisterCity {
        name: "Super City".to_string(),
        power_level: 5,
      },
    )
    .unwrap();

    let bad_city_creation = execute(
      deps.as_mut(),
      mock_env(),
      mock_info("mad", &[]),
      ExecuteMsg::RegisterCity {
        name: "Mad City".to_string(),
        power_level: 7,
      },
    );
    assert!(bad_city_creation.is_err());

    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("creator", &[]),
      ExecuteMsg::RegisterCity {
        name: "Secret City".to_string(),
        power_level: 3,
      },
    )
    .unwrap();

    let cities: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookCities { start_id: 0, limit: 10 },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      cities,
      ResponseMsg::LookCities {
        cities: [
          CityResponse {
            id: 0,
            name: "Super City".to_string(),
            power_level: 5,
            population: 0
          },
          CityResponse {
            id: 1,
            name: "Secret City".to_string(),
            power_level: 3,
            population: 0
          },
        ]
        .to_vec()
      }
    );
  }

  #[test]
  fn person_registration() {
    let mut deps = mock_dependencies();

    let mut env = mock_env();
    env.block.height = 887;
    env.block.time = Timestamp::from_seconds(YEAR_IN_SECONDS * 52 + YEAR_IN_SECONDS / 2); // in middle of 2022

    instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), InstantiateMsg {}).unwrap();

    let bad_person_registration = execute(
      deps.as_mut(),
      env.clone(),
      mock_info("user", &[]),
      ExecuteMsg::RegisterPerson {
        birthday: Birthday { day: None, year: 1755 },
        nickname: "super_user".to_string(),
        email: None,
      },
    );
    assert!(bad_person_registration.is_err());

    let bad_person_registration = execute(
      deps.as_mut(),
      env.clone(),
      mock_info("user", &[]),
      ExecuteMsg::RegisterPerson {
        birthday: Birthday { day: None, year: 2023 },
        nickname: "super_user".to_string(),
        email: None,
      },
    );
    assert!(bad_person_registration.is_err());

    let bad_person_registration = execute(
      deps.as_mut(),
      env.clone(),
      mock_info("user", &[]),
      ExecuteMsg::RegisterPerson {
        birthday: Birthday {
          day: Some(0),
          year: 2000,
        },
        nickname: "super_user".to_string(),
        email: None,
      },
    );
    assert!(bad_person_registration.is_err());

    let bad_person_registration = execute(
      deps.as_mut(),
      env.clone(),
      mock_info("user", &[]),
      ExecuteMsg::RegisterPerson {
        birthday: Birthday {
          day: Some(367),
          year: 2000,
        },
        nickname: "super_user".to_string(),
        email: None,
      },
    );
    assert!(bad_person_registration.is_err());

    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_1", &[]),
      ExecuteMsg::RegisterPerson {
        birthday: Birthday { day: None, year: 2000 },
        nickname: "super_user_1".to_string(),
        email: None,
      },
    )
    .unwrap();

    let bad_person_registration = execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_1", &[]),
      ExecuteMsg::RegisterPerson {
        birthday: Birthday { day: None, year: 2000 },
        nickname: "super_user_1".to_string(),
        email: None,
      },
    );
    assert!(bad_person_registration.is_err());

    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_2", &[]),
      ExecuteMsg::RegisterPerson {
        birthday: Birthday { day: None, year: 2009 },
        nickname: "super_user_2".to_string(),
        email: None,
      },
    )
    .unwrap();
  }

  #[test]
  fn person_updation() {
    let mut deps = mock_dependencies();

    instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), InstantiateMsg {}).unwrap();

    let bad_person_updation = execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user", &[]),
      ExecuteMsg::UpdatePerson {
        nickname: "super_user_1".to_string(),
        email: None,
      },
    );
    assert!(bad_person_updation.is_err());

    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user", &[]),
      ExecuteMsg::RegisterPerson {
        birthday: Birthday { day: None, year: 2000 },
        nickname: "super_user".to_string(),
        email: None,
      },
    )
    .unwrap();

    let person: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookPerson {
          person: Addr::unchecked("user"),
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      person,
      ResponseMsg::LookPerson {
        person: PersonResponse {
          address: Addr::unchecked("user"),
          birthday: Birthday { day: None, year: 2000 },
          nickname: "super_user".to_string(),
          email: None,
          resident_times: 0
        }
      }
    );

    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user", &[]),
      ExecuteMsg::UpdatePerson {
        nickname: "super_puper_user".to_string(),
        email: Some("user@user.io".to_string()),
      },
    )
    .unwrap();

    let person: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookPerson {
          person: Addr::unchecked("user"),
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      person,
      ResponseMsg::LookPerson {
        person: PersonResponse {
          address: Addr::unchecked("user"),
          birthday: Birthday { day: None, year: 2000 },
          nickname: "super_puper_user".to_string(),
          email: Some("user@user.io".to_string()),
          resident_times: 0
        }
      }
    );
  }

  #[test]
  fn person_in_city_registration() {
    let mut deps = mock_dependencies();

    instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), InstantiateMsg {}).unwrap();
    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("creator", &[]),
      ExecuteMsg::RegisterCity {
        name: "Super City".to_string(),
        power_level: 3,
      },
    )
    .unwrap();
    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("creator", &[]),
      ExecuteMsg::RegisterCity {
        name: "Secret City".to_string(),
        power_level: 3,
      },
    )
    .unwrap();

    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_1", &[]),
      ExecuteMsg::RegisterPerson {
        birthday: Birthday { day: None, year: 2000 },
        nickname: "super_user_1".to_string(),
        email: None,
      },
    )
    .unwrap();
    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_2", &[]),
      ExecuteMsg::RegisterPerson {
        birthday: Birthday { day: None, year: 2009 },
        nickname: "super_user_2".to_string(),
        email: None,
      },
    )
    .unwrap();

    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_1", &[]),
      ExecuteMsg::RegisterInCity { city_id: 0 },
    )
    .unwrap();

    let bad_registring_in_city = execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user", &[]),
      ExecuteMsg::RegisterInCity { city_id: 0 },
    );
    assert!(bad_registring_in_city.is_err());

    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_2", &[]),
      ExecuteMsg::RegisterInCity { city_id: 0 },
    )
    .unwrap();
    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_2", &[]),
      ExecuteMsg::RegisterInCity { city_id: 1 },
    )
    .unwrap();

    let city_people: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookCityPeople {
          city: 0,
          start_id: 0,
          limit: 10,
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      city_people,
      ResponseMsg::LookCityPeople {
        people: [
          PersonResponse {
            address: Addr::unchecked("user_1"),
            birthday: Birthday { day: None, year: 2000 },
            nickname: "super_user_1".to_string(),
            email: None,
            resident_times: 1
          },
          PersonResponse {
            address: Addr::unchecked("user_2"),
            birthday: Birthday { day: None, year: 2009 },
            nickname: "super_user_2".to_string(),
            email: None,
            resident_times: 2
          }
        ]
        .to_vec()
      }
    );

    let city_people: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookCityPeople {
          city: 1,
          start_id: 0,
          limit: 10,
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      city_people,
      ResponseMsg::LookCityPeople {
        people: [PersonResponse {
          address: Addr::unchecked("user_2"),
          birthday: Birthday { day: None, year: 2009 },
          nickname: "super_user_2".to_string(),
          email: None,
          resident_times: 2
        }]
        .to_vec()
      }
    );

    let person_cities: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookPersonCities {
          person: Addr::unchecked("user_1"),
          start_id: 0,
          limit: 10,
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      person_cities,
      ResponseMsg::LookPersonCities {
        cities: [CityResponse {
          id: 0,
          name: "Super City".to_string(),
          power_level: 3,
          population: 2
        }]
        .to_vec()
      }
    );

    let person_cities: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookPersonCities {
          person: Addr::unchecked("user_2"),
          start_id: 0,
          limit: 10,
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      person_cities,
      ResponseMsg::LookPersonCities {
        cities: [
          CityResponse {
            id: 0,
            name: "Super City".to_string(),
            power_level: 3,
            population: 2
          },
          CityResponse {
            id: 1,
            name: "Secret City".to_string(),
            power_level: 3,
            population: 1
          },
        ]
        .to_vec()
      }
    );

    let bad_registring_in_city = execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_1", &[]),
      ExecuteMsg::RegisterInCity { city_id: 0 },
    );
    assert!(bad_registring_in_city.is_err());

    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_1", &[]),
      ExecuteMsg::UnregisterFromCity { city_id: 0 },
    )
    .unwrap();
    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_1", &[]),
      ExecuteMsg::RegisterInCity { city_id: 0 },
    )
    .unwrap();

    let city_people: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookCityPeople {
          city: 0,
          start_id: 0,
          limit: 10,
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      city_people,
      ResponseMsg::LookCityPeople {
        people: [
          PersonResponse {
            address: Addr::unchecked("user_2"),
            birthday: Birthday { day: None, year: 2009 },
            nickname: "super_user_2".to_string(),
            email: None,
            resident_times: 2
          },
          PersonResponse {
            address: Addr::unchecked("user_1"),
            birthday: Birthday { day: None, year: 2000 },
            nickname: "super_user_1".to_string(),
            email: None,
            resident_times: 1
          }
        ]
        .to_vec()
      }
    );

    let city_people: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookCityPeople {
          city: 1,
          start_id: 0,
          limit: 10,
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      city_people,
      ResponseMsg::LookCityPeople {
        people: [PersonResponse {
          address: Addr::unchecked("user_2"),
          birthday: Birthday { day: None, year: 2009 },
          nickname: "super_user_2".to_string(),
          email: None,
          resident_times: 2
        }]
        .to_vec()
      }
    );

    let person_cities: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookPersonCities {
          person: Addr::unchecked("user_1"),
          start_id: 0,
          limit: 10,
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      person_cities,
      ResponseMsg::LookPersonCities {
        cities: [CityResponse {
          id: 0,
          name: "Super City".to_string(),
          power_level: 3,
          population: 2
        }]
        .to_vec()
      }
    );

    let person_cities: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookPersonCities {
          person: Addr::unchecked("user_2"),
          start_id: 0,
          limit: 10,
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      person_cities,
      ResponseMsg::LookPersonCities {
        cities: [
          CityResponse {
            id: 0,
            name: "Super City".to_string(),
            power_level: 3,
            population: 2
          },
          CityResponse {
            id: 1,
            name: "Secret City".to_string(),
            power_level: 3,
            population: 1
          },
        ]
        .to_vec()
      }
    );

    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_2", &[]),
      ExecuteMsg::UnregisterFromCity { city_id: 1 },
    )
    .unwrap();

    let city_people: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookCityPeople {
          city: 0,
          start_id: 0,
          limit: 10,
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      city_people,
      ResponseMsg::LookCityPeople {
        people: [
          PersonResponse {
            address: Addr::unchecked("user_2"),
            birthday: Birthday { day: None, year: 2009 },
            nickname: "super_user_2".to_string(),
            email: None,
            resident_times: 1
          },
          PersonResponse {
            address: Addr::unchecked("user_1"),
            birthday: Birthday { day: None, year: 2000 },
            nickname: "super_user_1".to_string(),
            email: None,
            resident_times: 1
          }
        ]
        .to_vec()
      }
    );

    let city_people: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookCityPeople {
          city: 1,
          start_id: 0,
          limit: 10,
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(city_people, ResponseMsg::LookCityPeople { people: [].to_vec() });

    let person_cities: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookPersonCities {
          person: Addr::unchecked("user_1"),
          start_id: 0,
          limit: 10,
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      person_cities,
      ResponseMsg::LookPersonCities {
        cities: [CityResponse {
          id: 0,
          name: "Super City".to_string(),
          power_level: 3,
          population: 2
        }]
        .to_vec()
      }
    );

    let person_cities: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookPersonCities {
          person: Addr::unchecked("user_2"),
          start_id: 0,
          limit: 10,
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      person_cities,
      ResponseMsg::LookPersonCities {
        cities: [CityResponse {
          id: 0,
          name: "Super City".to_string(),
          power_level: 3,
          population: 2
        }]
        .to_vec()
      }
    );

    let bad_upregistring_from_city = execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_2", &[]),
      ExecuteMsg::UnregisterFromCity { city_id: 1 },
    );
    assert!(bad_upregistring_from_city.is_err());

    let bad_upregistring_from_city = execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_2", &[]),
      ExecuteMsg::UnregisterFromCity { city_id: 2 },
    );
    assert!(bad_upregistring_from_city.is_err());

    let bad_registring_in_city = execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_1", &[]),
      ExecuteMsg::RegisterInCity { city_id: 3 },
    );
    assert!(bad_registring_in_city.is_err());

    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_1", &[]),
      ExecuteMsg::RegisterInCity { city_id: 1 },
    )
    .unwrap();
    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_1", &[]),
      ExecuteMsg::UnregisterFromCity { city_id: 0 },
    )
    .unwrap();
    execute(
      deps.as_mut(),
      mock_env(),
      mock_info("user_1", &[]),
      ExecuteMsg::UnregisterFromCity { city_id: 1 },
    )
    .unwrap();

    let city_people: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookCityPeople {
          city: 0,
          start_id: 0,
          limit: 10,
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      city_people,
      ResponseMsg::LookCityPeople {
        people: [PersonResponse {
          address: Addr::unchecked("user_2"),
          birthday: Birthday { day: None, year: 2009 },
          nickname: "super_user_2".to_string(),
          email: None,
          resident_times: 1
        }]
        .to_vec()
      }
    );

    let city_people: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookCityPeople {
          city: 1,
          start_id: 0,
          limit: 10,
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(city_people, ResponseMsg::LookCityPeople { people: [].to_vec() });

    let person_cities: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookPersonCities {
          person: Addr::unchecked("user_1"),
          start_id: 0,
          limit: 10,
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(person_cities, ResponseMsg::LookPersonCities { cities: [].to_vec() });

    let person_cities: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookPersonCities {
          person: Addr::unchecked("user_2"),
          start_id: 0,
          limit: 10,
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      person_cities,
      ResponseMsg::LookPersonCities {
        cities: [CityResponse {
          id: 0,
          name: "Super City".to_string(),
          power_level: 3,
          population: 1
        }]
        .to_vec()
      }
    );

    let cities: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookCities { start_id: 0, limit: 10 },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      cities,
      ResponseMsg::LookCities {
        cities: [
          CityResponse {
            id: 0,
            name: "Super City".to_string(),
            power_level: 3,
            population: 1
          },
          CityResponse {
            id: 1,
            name: "Secret City".to_string(),
            power_level: 3,
            population: 0
          },
        ]
        .to_vec()
      }
    );

    let person: ResponseMsg = from_binary(
      &query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LookPerson {
          person: Addr::unchecked("user_2"),
        },
      )
      .unwrap(),
    )
    .unwrap();
    assert_eq!(
      person,
      ResponseMsg::LookPerson {
        person: PersonResponse {
          address: Addr::unchecked("user_2"),
          birthday: Birthday { day: None, year: 2009 },
          nickname: "super_user_2".to_string(),
          email: None,
          resident_times: 1
        }
      }
    );
  }
}
