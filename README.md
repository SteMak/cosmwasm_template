# CosmWasm Template
The project aims to show CosmWasm features and highlight important points

- [CosmWasm Template](#cosmwasm-template)
  - [Development environment](#development-environment)
    - [Docker](#docker)
    - [Console tools](#console-tools)
    - [Rust 1.55.0+ (optional)](#rust-1550-optional)
    - [Wasmd (optional)](#wasmd-optional)
  - [Architecture](#architecture)
    - [System cache](#system-cache)
    - [Configuration](#configuration)
    - [Outputs](#outputs)
    - [Docker](#docker-1)
    - [Code](#code)
    - [Entrypoint](#entrypoint)
  - [Makefile entrypoints](#makefile-entrypoints)
    - [General](#general)
    - [Contract code](#contract-code)
    - [Chain actions](#chain-actions)
  - [Code notes](#code-notes)
    - [Derive](#derive)
    - [Custom errors](#custom-errors)
    - [Error propagation](#error-propagation)
    - [Addresses](#addresses)
    - [Contract entrypoints](#contract-entrypoints)
    - [Storage layout](#storage-layout)
  - [Getting started](#getting-started)
  - [Functional requirements](#functional-requirements)
    - [General overview and terms](#general-overview-and-terms)
    - [Instantiate method](#instantiate-method)
    - [Execute (set data) methods](#execute-set-data-methods)
    - [Query (view data) methods](#query-view-data-methods)


## Development environment
### Docker
Docker is the main tool you need:
[Official installation link](https://docs.docker.com/engine/install)

### Console tools
`jq`, `curl`, `make`, `sha3sum`, `tput`, `cat`, `cut` and some other common known tools that are used in `Makefile` 

### Rust 1.55.0+ (optional)
Optionally, you can manually install `Rust` on your system following the guide

Rust is needed as it is the main language for CosmWasm smart contracts developing

Install [rustup](https://rustup.rs/) if it is missed
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

It is considered to use stable compiler release
```bash
rustup default stable
```

Check compiler version and update if needed
```bash
cargo version # run next line if version is lower than 1.55.0+
rustup update stable
```

Install wasm32 target if it is missed
```bash
rustup target add wasm32-unknown-unknown
```

### Wasmd (optional)
Optionally, you can manually install `Wasmd` on your system following the guide

Wasmd is the tool for interacting with blockchain and smart contracts

Install [go](https://go.dev/doc/install) if it is missed

Clone the project
```bash
git clone https://github.com/CosmWasm/wasmd.git
cd wasmd
```

Fetch releases and switch to latest release compatible release
```bash
git fetch --tags
# replace version if latest release is not compatible 
git checkout $(git describe --tags `git rev-list --tags --max-count=1`)
```

Compile it!
```bash
make install
make build
```

Put the compiled file to $PATH
```bash
cp build/wasmd ~/.local/bin
```


## Architecture
### System cache
- `.cargo_analyzer/` - `cargo` target directory for `rust-analyzer` tool in `vscode`, it is different from default target dir as build cache from globally installed `rust` should not conflict with dockerized instance
- `.cargo_cache/` - `cargo` home directory, it stores all `cargo` dependencies mentioned in `Cargo.toml` file
- `.cargo_target/` - `cargo` target directory for dockerized `rust` instance
- `.wasmd_data/` - `wasmd` home directory, it stores all `wasmd` wallets, configuration, and cache

### Configuration
- `.cargo/` - directory stores `rust` configuration and `cargo` aliases
- `.vscode/` - directory stores `vscode` tools configurations
- `.editorconfig` - file stores general editor config
- `.gitignore` - file stores list of files ignored by `git`
- `rustfmt.toml` - configuration file for `rust` formatter

### Outputs
- `artifacts/` - directory for results of optimized build
- `coverage/` - directory for coverage report
- `schema/` - directory for JSON contract schema used on frontend

### Docker
- `docker_rust/` - directory stores some scripts needed in `rust` container
- `docker_wasmd/` - directory stores some scripts needed in `wasmd` container
- `Dockerfile.rust` - configuration file for `rust` container
- `Dockerfile.wasmd` - configuration file for `wasmd` container

### Code
- `examples/` - directory stores useful `rust` scripts that could be run separately from the whole project
- `examples/schema.rs` - script generates JSON schema of the contract
- `src/` - directory stores project source files
- `src/contract.rs` - source file stores contract entrypoints, execute/query methods and unit tests
- `src/error.rs` - source file stores expanded list of contract errors
- `src/lib.rs` - source file stores list of modules united to the library
- `src/msg.rs` - source file stores execute/query messages structs
- `src/state.rs` - source file stores storage layout and help functions
- `src/utils.rs` - source file stores some structs, types, and general functions
- `tests/` - directory stores test scripts
- `tests/integration.rs` - test file for verifying cross contract calls and other chain features

### Entrypoint
- `Makefile` - file stores all command aliases, detailed description is provided in corresponding section


## Makefile entrypoints
### General
- `setup` - build and configure `docker` images, it should be run once on project setup

### Contract code
- `code.build` - build contract code, the output file is not optimized enough for deploying to chain, but it may be used for running integration tests
- `code.build.optimize` - build contract code for deploying to chain
- `code.test.integration` - run integration tests, firstly, a wasm file should be built
- `code.test.unit` - run unit tests provided in contract file
- `code.test.coverage` - calculate unit tests coverage

### Chain actions
- `chain.wallet` - create and fund wallet, accept wallet name `wallet` parameter
- `chain.store_wasm` - load and check wasm code to chain, accept wallet name `wallet` and path to wasm `wasm` parameters
- `chain.contract.instantiate` - instantiate contract, accept wallet name `wallet`, instantiate message `msg` and optional code ID `code_id` parameters
- `chain.contract.execute` - execute message on contract, accept wallet name `wallet`, execute message `msg` and optional contract address `contract` parameters
- `chain.contract.query` - query data from contract, accept query message `msg` and optional contract address `contract` parameters


## Code notes
### Derive
```rs
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)] 
```
Derive implements features for a following structure:
- Serialize - adds possibility to code the struct into JSON (is needed when other contracts try to get the storage)
- Deserialize - adds possibility to decode the struct from JSON
- Clone - adds possibility to create duplicate of the struct by calling struct_instance.clone()
- Debug - adds possibility to use the structure in asserts
- PartialEq - adds possibility to compare instances of the struct
- JsonSchema - adds possibility to create JSON schema of the struct

### Custom errors
Custom errors are cool! You may specify your own error text with parameters
```rs
pub enum CustomError {
  #[error("Your access level is {have:?} and {needed:?} is needed")]
  Unauthorized { have: u8, needed: u8 },
}
```

### Error propagation
What it is going there?
```rs
Ok(to_binary(&query(get_part_param()? + another_part_param)?)?)
```
Error propagation is a great pattern as errors could be processed in one place avoiding panic in local functions

There is useful `Result<Ok_Type, Err_Type>` type defined, return values are unwrapped by `?` syntax

`?` works like unwrap, but does not panic on error, just propagate it to higher level

`?` may convert error type, if result error type implements `Std()` entry, for example:
```rs
pub enum CustomError {
  #[error("{0}")]
  Std(#[from] StdError),
}
```

### Addresses
There are `Addr` and `CanonicalAddr` types provided for addresses

`Addr` is based on `String` type with some validations and may be received as parameter, caller address `info.sender`, etc

`CanonicalAddr` is binary type and it is important to store any addresses in contract storage only wrapped to the type as text represetation may be changed in future

`String` - it is totally bad idea to store or manipulate addresses wrapped to the type

### Contract entrypoints
```rs
#[cfg_attr(not(feature = "library"), entry_point)]
```
Contract has 3 main entrypoints:
- `Instantiate` - should be called once on contract deployment, base storage layout is defined there
- `Execute` - base set data method, routes other methods
- `Query` - base view data method, routes other methods

As `execute` and `query` entrypoints are routers, signatures are defined separately in `msg.rs`
```rs
#[serde(rename_all = "snake_case")]
```
It is considered using snake case in json message field names

### Storage layout
```rs
Item::new("item_key");
Map::new("map_key");
```
CosmWasm implements key-value storage API, you should set unique keys manually for each storage instace

`Item` and `Map` are main storage types, desciption with examples on [crates.io](https://crates.io/crates/cw-storage-plus)


## Getting started


## Functional requirements
### General overview and terms
Maintainer is a contract manager with the highest access level

It is possible for anyone to become maintainer in some surcumstances

There are 2 main object groups: `People` and `Cities`

Contract maintainer is able to create a `City`

Anyone is able to create and update `Person`

Anyone is able to register/unregister his `Person` in/from any `City`

### Instantiate method
Set caller maintainer

Signature: `void`

Fail conditions: `void`

Return: `void`

### Execute (set data) methods
**RegisterCity**

Add new `City` providing metadata

Signature:
- `name: CityName` - part of `City` metadata
- `power_level: u8` - part of `City` metadata

Fail conditions:
- `Unauthorized` - caller is not maintainer

Return: `void`

**RegisterPerson**

Add new `Person` providing metadata

Signature:
- `birthday: Birthday` - part of `Person` metadata
- `nickname: Nickname` - part of `Person` metadata
- `email: Option<Email>` - part of `Person` metadata

Fail conditions:
- `InconsistentData` - `!(1 <= birthday.day <= 366)`
- `InconsistentData` - `!(1756 <= birthday.year <= current year)`
- `PersonAlreadyRegistered` - caller already created a `Person`

Return: `void`

**UpdatePerson**

Update `Person` metadata

Signature:
- `nickname: Nickname` - part of `Person` metadata
- `email: Option<Email>` - part of `Person` metadata

Fail conditions:
- `NotFound` - no `Person` created by caller found

Return: `void`

**RegisterInCity**

Register `Person` in `City`

Signature:
- `city_id: u64` - `City` identifier

Fail conditions:
- `NotFound` - no `Person` created by caller found
- `NotFound` - no `City` with the identifier found
- `PersonAlreadyRegisteredInCity` - `Person` is already registered in the `City`

Return: `void`

**UnregisterFromCity**

Unregister `Person` from `City`

Signature:
- `city_id: u64` - `City` identifier

Fail conditions:
- `NotFound` - no `Person` created by caller found
- `NotFound` - no `City` with the identifier found
- `NotFound` - `Person` is not registered in the `City`

Return: `void`

**BecomeMaintainer**

Set caller maintainer

Signature: `void`

Fail conditions:
- `AlreadyMaintainer` - caller is maintainer
- `NotFound` - no `Person` created by caller found
- `InconsistentMaintainer` - `Person` nickname is not `Super_Maintainer_887`
- `InconsistentMaintainer` - `Person` age is under `17`

Return: `void`

### Query (view data) methods
**LookMaintainer**

Check who is maintainer

Signature: `void`

Fail conditions: `void`

Return:
- `maintainer: Addr` - maintainer address

**LookPerson**

Check `Person` metadata

Signature:
- `person: Addr` - address of user created `Person`

Fail conditions:
- `NotFound` - no `Person` created by queried address found

Return:
- `person: PersonResponse` -
  - `address: Addr` - queried address
  - `birthday: Birthday` - part of `Person` metadata
  - `nickname: Nickname` - part of `Person` metadata
  - `email: Option<Email>` - part of `Person` metadata
  - `resident_times: u64` - amount of `Cities` where `Person` is registered

**LookCities**

Check `Cities` list with metadata

Signature:
- `start_id: u64` - start `City` identifier
- `limit: u64` - maximum amount of `Cities` responsed

Fail conditions: `void`

Return:
- `cities: Vec<CityResponse>` -
  - `id: u64` - `City` identifier
  - `name: CityName` - part of `City` metadata
  - `power_level: u8` - part of `City` metadata
  - `population: u64` - amount of `People` registred in the `City`

**LookPersonCities**

Check `Cities` list with metadata where the `Person` is registered

Signature:
- `person: Addr` - address of user created `Person`
- `start_id: u64` - start `City` identifier
- `limit: u64` - maximum amount of `Cities` responsed

Fail conditions:
- `NotFound` - no `Person` created by queried address found

Return:
- `cities: Vec<CityResponse>` -
  - `id: u64` - `City` identifier
  - `name: CityName` - part of `City` metadata
  - `power_level: u8` - part of `City` metadata
  - `population: u64` - amount of `People` registred in the `City`

**LookCityPeople**

Check `People` metadata by city where they are registered

Signature:
- `city: u64` - `City` identifier
- `start_id: u64` - start `Person` identifier
- `limit: u64` - maximum amount of `People` responsed

Fail conditions:
- `NotFound` - no `City` with the identifier found

Return:
- `people: Vec<PersonResponse>` -
  - `address: Addr` - queried address
  - `birthday: Birthday` - part of `Person` metadata
  - `nickname: Nickname` - part of `Person` metadata
  - `email: Option<Email>` - part of `Person` metadata
  - `resident_times: u64` - amount of `Cities` where `Person` is registered
