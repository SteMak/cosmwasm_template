CONTAINER_RUST=cosmwasm_rust
CONTAINER_WASMD=cosmwasm_wasmd
CURRENT_UID=$(shell id -u)
DIR=$(shell pwd)

CHAIN_FAUCET=https://faucet.malaga-420.cosmwasm.com
CHAIN_FEE_DENOM=umlg
CHAIN_ID="malaga-420"
CHAIN_TXFLAG="--chain-id $(CHAIN_ID) --gas-prices 0.25$(FEE_DENOM) --gas auto --gas-adjustment 1.3"

--docker_container_build_rust:
	docker build -t $(CONTAINER_RUST) --build-arg UID=$(CURRENT_UID) -f $(DIR)/Dockerfile.rust $(DIR)/docker_rust

--docker_container_build_wasmd:
	docker build -t $(CONTAINER_WASMD) --build-arg UID=$(CURRENT_UID) -f $(DIR)/Dockerfile.wasmd $(DIR)/docker_wasmd

--fetch_cargo_dependencies:
	docker run --rm --volume $(DIR):/usr/cosmwasm_docker $(CONTAINER_RUST) cargo fetch

--set_wasmd_config:
	mkdir -p $(DIR)/.wasmd_data
	docker run --rm --volume $(DIR)/.wasmd_data:/home/wasm_user/.wasmd $(CONTAINER_WASMD) wasmd config keyring-backend test

setup: \
	--docker_container_build_rust --fetch_cargo_dependencies \
	--docker_container_build_wasmd --set_wasmd_config

code.build:
	docker run --rm --volume $(DIR):/usr/cosmwasm_docker $(CONTAINER_RUST) cargo wasm
code.test.unit:
	docker run --rm --volume $(DIR):/usr/cosmwasm_docker $(CONTAINER_RUST) cargo unit-test -- --color=always
code.test.integration:
	docker run --rm --volume $(DIR):/usr/cosmwasm_docker $(CONTAINER_RUST) cargo integration-test -- --color=always
code.test.coverage:
	docker run --rm --volume $(DIR):/usr/cosmwasm_docker --security-opt seccomp=unconfined $(CONTAINER_RUST) cargo coverage --color=always
code.schema:
	docker run --rm --volume $(DIR):/usr/cosmwasm_docker $(CONTAINER_RUST) cargo schema

code.build.optimize:
	sudo rm -rf $(DIR)/.optimize_cache
	sudo rm -rf $(DIR)/artifacts

	mkdir -p $(DIR)/.optimize_cache
	cp $(DIR)/Cargo.toml $(DIR)/.optimize_cache
	cp $(DIR)/Cargo.lock $(DIR)/.optimize_cache
	cp -r $(DIR)/src $(DIR)/.optimize_cache/src

	docker run --rm \
		-e CARGO_TERM_COLOR=always \
		--volume $(DIR)/.optimize_cache:/code \
		--volume $(CONTAINER_RUST)_cache:/code/target \
		--volume registry_cache:/usr/local/cargo/registry \
		cosmwasm/rust-optimizer:0.12.6

	sudo chown -R $(CURRENT_UID) $(DIR)/.optimize_cache/artifacts
	mv $(DIR)/.optimize_cache/artifacts $(DIR)
	rm $(DIR)/artifacts/checksums_intermediate.txt

	sudo rm -rf $(DIR)/.optimize_cache


chain.wallet.create:
ifndef wallet
	$(error "Error: missed wallet name, try 'make chain.wallet.create wallet=WALLET_NAME'")
endif
	docker run --rm -it --volume $(DIR)/.wasmd_data:/home/wasm_user/.wasmd $(CONTAINER_WASMD) wasmd keys add $(wallet)

chain.wallet.fund:
ifndef wallet
	$(error "Error: missed wallet name, try 'make chain.wallet.fund wallet=WALLET_NAME'")
endif
	curl -X POST --header "Content-Type: application/json" \
		--data '{ "denom": "$(CHAIN_FEE_DENOM)", "address": \
			"$(shell docker run --rm --volume $(DIR)/.wasmd_data:/home/wasm_user/.wasmd $(CONTAINER_WASMD) wasmd keys show -a $(wallet))" \
		}' $(CHAIN_FAUCET)/credit

chain.wallet: chain.wallet.create chain.wallet.fund

chain.store_wasm.push_code:
ifndef wallet
	$(error "Error: missed wallet name, try 'make chain.store_wasm.push_code wallet=WALLET_NAME wasm=PATH_TO_WASM'")
endif
ifndef wasm
	$(error "Error: missed path to wasm, try 'make chain.store_wasm.push_code wallet=WALLET_NAME wasm=PATH_TO_WASM'")
endif
	docker run --rm \
		--volume $(PWD):/usr/scope \
		--volume $(DIR)/.wasmd_data:/home/wasm_user/.wasmd \
		$(CONTAINER_WASMD) \
	wasmd tx wasm store $(wasm) --from $(wallet) \
		--chain-id $(CHAIN_ID) --gas-prices 0.25$(FEE_DENOM) --gas auto --gas-adjustment 1.3 -y --output json -b block > $(DIR)/.wasmd_data/cached_code_loaded_json
	cat $(DIR)/.wasmd_data/cached_code_loaded_json | jq -r '.logs[0].events[-1].attributes[0].value' > $(DIR)/.wasmd_data/cached_code_loaded_id

chain.store_wasm.download_code:
	$(eval code_id := $(shell cat $(DIR)/.wasmd_data/cached_code_loaded_id 2>/dev/null))
	@if [ "$(code_id)" == "$(EMP)" ]; then \
		echo "Error: missing cached code id, load code first"; \
		exit 1; \
	fi

	@echo "Reading last loaded code id: $(code_id)"
	docker run --rm \
		--volume $(DIR)/.wasmd_data:/home/wasm_user/.wasmd \
		$(CONTAINER_WASMD) \
	wasmd query wasm code $(code_id) /home/wasm_user/.wasmd/cached_code.wasm

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
		echo "Error: loaded code differs from provided instance, try again"; \
		exit 1; \
	fi
	@echo "Code is loaded perfectly"

chain.store_wasm: chain.store_wasm.push_code chain.store_wasm.download_code chain.store_wasm.compare_code
	@rm $(DIR)/.wasmd_data/cached_code.wasm
	@rm $(DIR)/.wasmd_data/cached_code_loaded_json
	@rm $(DIR)/.wasmd_data/cached_code_loaded_id
