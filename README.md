# CosmWasm Template
The project aims to show CosmWasm features and highlight important points


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
- `setup` - build and configure docker images, it should be run once on project setup

### Contract code
- `code.build` - build contract code, the output file is not optimized enough for deploying to chain, but it needs for running integration tests
- `code.build.optimize` - build contract code for deploying to chain
- `code.test.integration` - run integration tests, firstly, a wasm file should be built
- `code.test.unit` - run unit tests provided in contract file
- `code.test.coverage` - calculate unit tests coverage and stores report in `coverage` folder

### Chain actions
- `chain.wallet` - create and fund wallet, accept wallet name `wallet` parameter
- `chain.store_wasm` - load and check wasm code to chain, accept wallet name `wallet` and path to wasm `wasm` parameters
- `chain.contract.instantiate` - instantiate contract, accept wallet name `wallet`, instantiate message `msg` and optional code ID `code_id` parameters
- `chain.contract.execute` - execute message on contract, accept wallet name `wallet`, execute message `msg` and optional contract address `contract` parameters
- `chain.contract.query` - query data from contract, accept query message `msg` and optional contract address `contract` parameters
