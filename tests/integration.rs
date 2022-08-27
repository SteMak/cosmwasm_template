// This integration test tries to run and call the generated wasm.
// It depends on a Wasm build being available, which you can create with `cargo wasm`.
// Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//
// You can easily convert unit tests to integration tests as follows:
// 1. Copy them over verbatim
// 2. Then change
//      let mut deps = mock_dependencies(20, &[]);
//    to
//      let mut deps = mock_instance(WASM, &[]);
// 3. If you access raw storage, where ever you see something like:
//      deps.storage.get(CONFIG_KEY).expect("no data stored");
//    replace it with:
//      deps.with_storage(|store| {
//          let data = store.get(CONFIG_KEY).expect("no data stored");
//      });
// 4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)

use cosmwasm_std::{
  from_binary, Addr, BlockInfo, ContractInfo, Env, MessageInfo, Response, Timestamp, TransactionInfo,
};
use cosmwasm_vm::testing::{execute, instantiate, mock_info, mock_instance, query};

use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;
use cosmwasm_template::{
  msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ResponseMsg},
  utils::{Birthday, CityResponse, PersonResponse},
};

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../.cargo_target/wasm32-unknown-unknown/release/cosmwasm_template.wasm");
// You can uncomment this line instead to test productionified build from rust-optimizer
// static WASM: &[u8] = include_bytes!("../artifacts/cosmwasm_template.wasm");

fn mock_env_info_height(signer: &str, height: u64, time: u64) -> (Env, MessageInfo) {
  let env = Env {
    block: BlockInfo {
      height,
      time: Timestamp::from_seconds(time),
      chain_id: String::from("test"),
    },
    contract: ContractInfo {
      address: Addr::unchecked(MOCK_CONTRACT_ADDR),
    },
    transaction: Some(TransactionInfo { index: 7 }),
  };
  let info = mock_info(signer, &[]);
  (env, info)
}

#[test]
fn initialization() {
  let mut deps = mock_instance(WASM, &[]);
  let msg = InstantiateMsg {};
  let (env, info) = mock_env_info_height("creator", 887, 31556952 * 52 + 31556952 / 2); // in middle of 2022
  let res: Response = instantiate(&mut deps, env.clone(), info, msg).unwrap();
  assert_eq!(0, res.messages.len());

  let maintainer: ResponseMsg =
    from_binary(&query(&mut deps, env.clone(), QueryMsg::LookMaintainer {}).unwrap()).unwrap();
  assert_eq!(
    maintainer,
    ResponseMsg::LookMaintainer {
      maintainer: Addr::unchecked("creator")
    }
  );
}

#[test]
fn person_in_city_registration() {
  let mut deps = mock_instance(WASM, &[]);
  let msg = InstantiateMsg {};
  let (env_1, info_1) = mock_env_info_height("creator", 887, 31556952 * 52 + 31556952 / 2);
  let (env_2, info_2) = mock_env_info_height("user_2", 887, 31556952 * 52 + 31556952 / 2);
  let (env_3, info_3) = mock_env_info_height("user_3", 887, 31556952 * 52 + 31556952 / 2);

  let _: Response = instantiate(&mut deps, env_1.clone(), info_1.clone(), msg).unwrap();

  let _: Response = execute(
    &mut deps,
    env_1.clone(),
    info_1.clone(),
    ExecuteMsg::RegisterCity {
      name: "Super City".to_string(),
      power_level: 3,
    },
  )
  .unwrap();
  let _: Response = execute(
    &mut deps,
    env_1.clone(),
    info_1.clone(),
    ExecuteMsg::RegisterCity {
      name: "Secret City".to_string(),
      power_level: 3,
    },
  )
  .unwrap();

  let _: Response = execute(
    &mut deps,
    env_2.clone(),
    info_2.clone(),
    ExecuteMsg::RegisterPerson {
      birthday: Birthday { day: None, year: 2000 },
      nickname: "super_user_2".to_string(),
      email: None,
    },
  )
  .unwrap();
  let _: Response = execute(
    &mut deps,
    env_3.clone(),
    info_3.clone(),
    ExecuteMsg::RegisterPerson {
      birthday: Birthday { day: None, year: 2009 },
      nickname: "super_user_3".to_string(),
      email: None,
    },
  )
  .unwrap();

  let _: Response = execute(
    &mut deps,
    env_2.clone(),
    info_2.clone(),
    ExecuteMsg::RegisterInCity { city_id: 0 },
  )
  .unwrap();

  let _: Response = execute(
    &mut deps,
    env_3.clone(),
    info_3.clone(),
    ExecuteMsg::RegisterInCity { city_id: 0 },
  )
  .unwrap();
  let _: Response = execute(
    &mut deps,
    env_3.clone(),
    info_3.clone(),
    ExecuteMsg::RegisterInCity { city_id: 1 },
  )
  .unwrap();

  let city_people: ResponseMsg = from_binary(
    &query(
      &mut deps,
      env_1.clone(),
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
          birthday: Birthday { day: None, year: 2000 },
          nickname: "super_user_2".to_string(),
          email: None,
          resident_times: 1
        },
        PersonResponse {
          address: Addr::unchecked("user_3"),
          birthday: Birthday { day: None, year: 2009 },
          nickname: "super_user_3".to_string(),
          email: None,
          resident_times: 2
        }
      ]
      .to_vec()
    }
  );

  let city_people: ResponseMsg = from_binary(
    &query(
      &mut deps,
      env_2.clone(),
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
        address: Addr::unchecked("user_3"),
        birthday: Birthday { day: None, year: 2009 },
        nickname: "super_user_3".to_string(),
        email: None,
        resident_times: 2
      }]
      .to_vec()
    }
  );

  let person_cities: ResponseMsg = from_binary(
    &query(
      &mut deps,
      env_2.clone(),
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

  let person_cities: ResponseMsg = from_binary(
    &query(
      &mut deps,
      env_2.clone(),
      QueryMsg::LookPersonCities {
        person: Addr::unchecked("user_3"),
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
}
