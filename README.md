# CosmWasm Template
The project aims to show CosmWasm features and highlight important points.


## Development environment
### Docker
Docker is main tool you need:
[Official installation link](https://docs.docker.com/engine/install)

### Console tools
`jq`, `curl`, `make`, `sha3sum`, `tput`, `cat`, `cut` and some other common known tools that are used in `Makefile` 

### Rust 1.55.0+ (optional)
Optionally you can manually install `Rust` on your system following the guide

Rust is needed as it is the main language for CosmWasm smart contracts developing

Install [rustup](https://rustup.rs/) if it is missed
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

It is considered to use stable compiler release
```bash
rustup default stable
```

Check compiler version und update if needed
```bash
cargo version # run next line if version is lower than 1.55.0+
rustup update stable
```

Install wasm32 target if it is missed
```bash
rustup target add wasm32-unknown-unknown
```

### Wasmd (optional)
Optionally you can manually install `Wasmd` on your system following the guide

Wasmd is tool for interacting with blockchain and smart contracts

Install [go](https://go.dev/doc/install) if it is missed

Clone the project
```bash
git clone https://github.com/CosmWasm/wasmd.git
cd wasmd
```

Fetch releases and switch to latest realese compatible release
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
