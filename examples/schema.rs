// The file contains script generates JSON schema of the contract
// The generated JSON schema may be needed on frontend for
// constructing valid messages and validating contract responses

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_template::{
  msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ResponseMsg},
  utils::Config,
};

fn main() {
  let mut out_dir = current_dir().unwrap();
  out_dir.push("schema");
  create_dir_all(&out_dir).unwrap();
  remove_schemas(&out_dir).unwrap();

  // List of message types
  export_schema(&schema_for!(InstantiateMsg), &out_dir);
  export_schema(&schema_for!(ExecuteMsg), &out_dir);
  export_schema(&schema_for!(QueryMsg), &out_dir);
  export_schema(&schema_for!(ResponseMsg), &out_dir);

  // Config schema may be also needed as it possible to get Items directly from storage
  export_schema(&schema_for!(Config), &out_dir);
}
