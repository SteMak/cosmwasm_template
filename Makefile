CONTAINER_RUST=cosmwasm_rust
CONTAINER_WASMD=cosmwasm_wasmd
CURRENT_UID=$(shell id -u)
DIR=$(shell pwd)

CHAIN_FAUCET=https://faucet.malaga-420.cosmwasm.com
CHAIN_FEE_DENOM=umlg
CHAIN_ID="malaga-420"
CHAIN_TXFLAG="--chain-id $(CHAIN_ID) --gas-prices 0.25$(CHAIN_FEE_DENOM) --gas auto --gas-adjustment 1.3"

# Build rust container
docker_container_build_rust:
	@docker build -t $(CONTAINER_RUST) --build-arg UID=$(CURRENT_UID) -f $(DIR)/Dockerfile.rust $(DIR)/docker_rust
	@tput setaf 2; echo "==> Dockerfile.rust built"; tput sgr0
	@echo

# Build wasmd container
docker_container_build_wasmd:
	@docker build -t $(CONTAINER_WASMD) --build-arg UID=$(CURRENT_UID) -f $(DIR)/Dockerfile.wasmd $(DIR)/docker_wasmd
	@tput setaf 2; echo "==> Dockerfile.wasmd built"; tput sgr0
	@echo

# Cache rust dependencies
fetch_cargo_dependencies:
	@docker run --rm --volume $(DIR):/usr/cosmwasm_docker $(CONTAINER_RUST) cargo fetch
	@tput setaf 2; echo "==> Cargo dependencies fetched"; tput sgr0
	@echo

# Create wasmd cache dir, configure keyring to test
set_wasmd_config:
	@mkdir -p $(DIR)/.wasmd_data
	@docker run --rm --volume $(DIR)/.wasmd_data:/home/wasm_user/.wasmd $(CONTAINER_WASMD) wasmd config keyring-backend test
	@tput setaf 2; echo "==> Wasmd configured"; tput sgr0
	@echo

# Setup docker containers
setup: \
	docker_container_build_rust fetch_cargo_dependencies \
	docker_container_build_wasmd set_wasmd_config

# Aliases to rust
code.build:
	@docker run --rm --volume $(DIR):/usr/cosmwasm_docker $(CONTAINER_RUST) cargo wasm
code.test.unit:
	@docker run --rm --volume $(DIR):/usr/cosmwasm_docker $(CONTAINER_RUST) cargo unit-test -- --color=always
code.test.integration:
	@docker run --rm --volume $(DIR):/usr/cosmwasm_docker $(CONTAINER_RUST) cargo integration-test -- --color=always
code.test.coverage:
	@docker run --rm --volume $(DIR):/usr/cosmwasm_docker --security-opt seccomp=unconfined $(CONTAINER_RUST) cargo coverage --color=always
code.schema:
	@docker run --rm --volume $(DIR):/usr/cosmwasm_docker $(CONTAINER_RUST) cargo schema

# Run optimize build with cosmwasm/rust-optimizer
code.build.optimize:
# Remove possible cache and artifacts dirs
	@sudo rm -rf $(DIR)/.optimize_cache
	@sudo rm -rf $(DIR)/artifacts
	@tput setaf 2; echo "==> Possible uncleared cache removed"; tput sgr0
	@echo

# Cache source files
	@mkdir -p $(DIR)/.optimize_cache
	@cp $(DIR)/Cargo.toml $(DIR)/.optimize_cache
	@cp $(DIR)/Cargo.lock $(DIR)/.optimize_cache
	@cp -r $(DIR)/src $(DIR)/.optimize_cache/src
	@tput setaf 2; echo "==> Source code copied"; tput sgr0
	@echo

# Run optimizer
	@docker run --rm \
		-e CARGO_TERM_COLOR=always \
		--volume $(DIR)/.optimize_cache:/code \
		--volume $(CONTAINER_RUST)_cache:/code/target \
		--volume registry_cache:/usr/local/cargo/registry \
		cosmwasm/rust-optimizer:0.12.6
	@tput setaf 2; echo "==> Optimized build finished"; tput sgr0
	@echo

# Chown artifacts dir (is needed if docker default user is root)
	@sudo chown -R $(CURRENT_UID) $(DIR)/.optimize_cache/artifacts

# Move result
	@mv $(DIR)/.optimize_cache/artifacts $(DIR)
	@rm $(DIR)/artifacts/checksums_intermediate.txt
	@tput setaf 2; echo "==> Artifacts copied"; tput sgr0
	@echo

# Remove cache
	@sudo rm -rf $(DIR)/.optimize_cache
	@tput setaf 2; echo "==> Cache removed"; tput sgr0
	@echo


# Create wasmd wallet
chain.wallet.create:
ifndef wallet
	$(error "Error: missed wallet name, try 'make chain.wallet.create wallet=WALLET_NAME'")
endif
	@docker run --rm -it --volume $(DIR)/.wasmd_data:/home/wasm_user/.wasmd $(CONTAINER_WASMD) wasmd keys add $(wallet)
	@tput setaf 2; echo "==> Wallet created"; tput sgr0
	@echo

# Fund wasmd wallet
chain.wallet.fund:
ifndef wallet
	$(error "Error: missed wallet name, try 'make chain.wallet.fund wallet=WALLET_NAME'")
endif
	$(eval address := $(shell docker run --rm --volume $(DIR)/.wasmd_data:/home/wasm_user/.wasmd $(CONTAINER_WASMD) wasmd keys show -a $(wallet)))
	@curl -X POST --header "Content-Type: application/json" --fail \
		--data '{ "denom": "$(CHAIN_FEE_DENOM)", "address": "$(address)" }' $(CHAIN_FAUCET)/credit
	@echo
	@tput setaf 2; echo "==> Wallet funded"; tput sgr0
	@echo

# Create and fund wallet
chain.wallet: chain.wallet.create chain.wallet.fund

# Load built code to chain
chain.store_wasm.push_code:
ifndef wallet
	$(error "Error: missed wallet name, try 'make chain.store_wasm.push_code wallet=WALLET_NAME wasm=PATH_TO_WASM'")
endif
ifndef wasm
	$(error "Error: missed path to wasm, try 'make chain.store_wasm.push_code wallet=WALLET_NAME wasm=PATH_TO_WASM'")
endif
	@docker run --rm \
		--volume $(PWD):/usr/scope \
		--volume $(DIR)/.wasmd_data:/home/wasm_user/.wasmd \
		$(CONTAINER_WASMD) \
	wasmd tx wasm store $(wasm) --from $(wallet) --chain-id $(CHAIN_ID) \
		--gas-prices 0.25$(CHAIN_FEE_DENOM) --gas auto --gas-adjustment 1.3 -y --output json -b block > $(DIR)/.wasmd_data/cached_code_resp
	@cat $(DIR)/.wasmd_data/cached_code_resp | jq -r '.logs[0].events[-1].attributes[0].value' > $(DIR)/.wasmd_data/cached_code_id
	@rm $(DIR)/.wasmd_data/cached_code_resp

	@tput setaf 2; echo "==> Code pushed"; tput sgr0
	@echo

# Download built code from chain
chain.store_wasm.download_code:
# Use cached code id if the parameter is missed
	$(eval cached_code_id := $(shell cat $(DIR)/.wasmd_data/cached_code_id 2>/dev/null))
	@if [ "$(code_id)" == "$(EMP)" ] && [ "$(cached_code_id)" == "$(EMP)" ]; then \
		echo "Error: missing cached code id or code_id parameter"; \
		exit 1; \
	fi
	@if [ "$(code_id)" == "$(EMP)" ]; then \
		echo "Reading last loaded code id: $(cached_code_id)"; \
		$(eval code_id = $(shell cat $(DIR)/.wasmd_data/cached_code_id 2>/dev/null)) \
	fi

	@docker run --rm \
		--volume $(DIR)/.wasmd_data:/home/wasm_user/.wasmd \
		$(CONTAINER_WASMD) \
	wasmd query wasm code $(code_id) /home/wasm_user/.wasmd/cached_code.wasm

	@tput setaf 2; echo "==> Code downloaded"; tput sgr0
	@echo

# Check if loaded code is the same as provided
chain.store_wasm.compare_code:
ifndef wasm
	$(error "Error: missed path to wasm, try 'make chain.store_wasm.compare_code wasm=PATH_TO_WASM code_id=CODE_ID'")
endif
	$(eval sha_1 := $(shell sha3sum $(DIR)/.wasmd_data/cached_code.wasm 2>/dev/null | cut -d " " -f 1))
	$(eval sha_2 := $(shell sha3sum $(wasm) 2>/dev/null | cut -d " " -f 1))

	@if [ "$(sha_1)" == "$(EMP)" ]; then \
		echo "Error: missing cached wasm file, download it first"; \
		exit 1; \
	fi
	@if [ "$(sha_2)" == "$(EMP)" ]; then \
		echo "Error: bad wasm file provided"; \
		exit 1; \
	fi
	@if [ "$(sha_1)" != "$(sha_2)" ]; then \
		echo "Error: downloaded code differs from provided instance, try again"; \
		exit 1; \
	fi
	@tput setaf 2; echo "==> Pushed and downloaded codes are equal"; tput sgr0
	@echo

# Load code to chain and check sha3 sums
chain.store_wasm: chain.store_wasm.push_code chain.store_wasm.download_code chain.store_wasm.compare_code

# Call instantiate message on contract
chain.contract.instantiate:
ifndef msg
	$(error "Error: missed msg, try 'make chain.contract.instantiate msg=MESSAGE wallet=WALLET_NAME'")
endif
ifndef wallet
	$(error "Error: missed wallet name, try 'make chain.contract.instantiate msg=MESSAGE wallet=WALLET_NAME'")
endif
# Use cached code id if the parameter is missed
	$(eval cached_code_id := $(shell cat $(DIR)/.wasmd_data/cached_code_id 2>/dev/null))
	@if [ "$(code_id)" == "$(EMP)" ] && [ "$(cached_code_id)" == "$(EMP)" ]; then \
		echo "Error: missing cached code id or code_id parameter"; \
		exit 1; \
	fi
	@if [ "$(code_id)" == "$(EMP)" ]; then \
		echo "Reading last loaded code id: $(cached_code_id)"; \
		$(eval code_id = $(shell cat $(DIR)/.wasmd_data/cached_code_id 2>/dev/null)) \
	fi

	@docker run --rm \
		--volume $(DIR)/.wasmd_data:/home/wasm_user/.wasmd \
		$(CONTAINER_WASMD) \
	wasmd tx wasm instantiate $(code_id) $(msg) --from $(wallet) --label "People & Cities" --chain-id $(CHAIN_ID) \
		--gas-prices 0.25$(CHAIN_FEE_DENOM) --gas auto --gas-adjustment 1.3 -y --no-admin --output json -b block > $(DIR)/.wasmd_data/cached_init_resp
	@cat $(DIR)/.wasmd_data/cached_init_resp | jq -r '.logs[0].events[0].attributes[0].value' > $(DIR)/.wasmd_data/cached_address
	@rm $(DIR)/.wasmd_data/cached_init_resp

	@tput setaf 2; echo "==> Contract instantiated"; tput sgr0
	@echo

# Call execute message on contract
chain.contract.execute:
ifndef msg
	$(error "Error: missed msg, try 'make chain.contract.execute msg=MESSAGE wallet=WALLET_NAME'")
endif
ifndef wallet
	$(error "Error: missed wallet name, try 'make chain.contract.execute msg=MESSAGE wallet=WALLET_NAME'")
endif
# Use cached contract adderss if the parameter is missed
	$(eval cached_contract := $(shell cat $(DIR)/.wasmd_data/cached_address 2>/dev/null))
	@if [ "$(contract)" == "$(EMP)" ] && [ "$(cached_contract)" == "$(EMP)" ]; then \
		echo "Error: missing cached address or contract parameter"; \
		exit 1; \
	fi
	@if [ "$(contract)" == "$(EMP)" ]; then \
		echo "Reading last instantiated contract: $(cached_contract)"; \
		$(eval contract = $(shell cat $(DIR)/.wasmd_data/cached_address 2>/dev/null)) \
	fi
	
	@docker run --rm \
		--volume $(DIR)/.wasmd_data:/home/wasm_user/.wasmd \
		$(CONTAINER_WASMD) \
	wasmd tx wasm execute $(contract) '$(msg)' --from $(wallet) --chain-id $(CHAIN_ID) \
		--gas-prices 0.25$(CHAIN_FEE_DENOM) --gas auto --gas-adjustment 1.3 -y --output json -b block > $(DIR)/.wasmd_data/cached_execute_resp
	@echo "Transaction hash: " > $(DIR)/.wasmd_data/cached_execute_hash
	@cat $(DIR)/.wasmd_data/cached_execute_resp | jq -r '.txhash' >> $(DIR)/.wasmd_data/cached_execute_hash
	@cat $(DIR)/.wasmd_data/cached_execute_hash | tr -d '\n'
	@echo
	@rm $(DIR)/.wasmd_data/cached_execute_resp
	@rm $(DIR)/.wasmd_data/cached_execute_hash

	@tput setaf 2; echo "==> Contract executed"; tput sgr0
	@echo

# Call query message on contract
chain.contract.query:
ifndef msg
	$(error "Error: missed msg, try 'make chain.contract.query msg=MESSAGE'")
endif
# Use cached contract adderss if the parameter is missed
	$(eval cached_contract := $(shell cat $(DIR)/.wasmd_data/cached_address 2>/dev/null))
	@if [ "$(contract)" == "$(EMP)" ] && [ "$(cached_contract)" == "$(EMP)" ]; then \
		echo "Error: missing cached address or contract parameter"; \
		exit 1; \
	fi
	@if [ "$(contract)" == "$(EMP)" ]; then \
		echo "Reading last instantiated contract: $(cached_contract)"; \
		$(eval contract = $(shell cat $(DIR)/.wasmd_data/cached_address 2>/dev/null)) \
	fi
	
	@docker run --rm \
		--volume $(DIR)/.wasmd_data:/home/wasm_user/.wasmd \
		$(CONTAINER_WASMD) \
	wasmd query wasm contract-state smart $(contract) '$(msg)'

	@tput setaf 2; echo "==> Contract queried"; tput sgr0
	@echo
