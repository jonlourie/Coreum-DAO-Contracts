COREUM_CHAIN_ID="coreum-devnet-1"
COREUM_DENOM=udevcore
COREUM_NODE=http://localhost:26657
COREUM_VERSION="{Cored version}"
COREUM_CHAIN_ID_ARGS=--chain-id=$(COREUM_CHAIN_ID)
COREUM_NODE_ARGS=--node=$(COREUM_NODE)
COREUM_HOME=$(HOME)/.core/"$(COREUM_CHAIN_ID)"
COREUM_BINARY_NAME=$(shell arch | sed s/aarch64/cored-linux-arm64/ | sed s/x86_64/cored-linux-amd64/)

DEV_WALLET=dev-wallet
CODE_ID=1
SUBUNIT=gil

# replace this after you instantiate your contract
_CONTRACT_ADDRESS_=devcore14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sd4f0ak
_WALLET_ADDRESS_=devcore17vlkqgpxnrqqrgq877v7a4ymfma0r03g02agg8
dev:
	echo "${PWD}"
	echo `basename "${PWD}"`
	cargo build
test:
	cargo test -- --nocapture
add_account:
	cored-00 keys add ${DEV_WALLET} --recover
build:
	docker run --rm -v "${PWD}":/code --mount type=volume,source=`basename "${PWD}"`_cache,target=/code/target --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry cosmwasm/rust-optimizer:0.14.0

deploy:
	RES=$$(cored-00 tx wasm store artifacts/grant-dao.wasm --from ${DEV_WALLET} --gas auto --gas-adjustment 1.3 -y -b block --output json "$(COREUM_NODE_ARGS)" "$(COREUM_CHAIN_ID_ARGS)") ; \
	echo $$RES ; \
	CODE_ID=$$(echo $$RES | jq -r '.logs[0].events[-1].attributes[-1].value') ; \
	echo "Code ID: $$CODE_ID"
check:
	cored-00 q wasm code-info $(CODE_ID) $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS)
keys:
	cored-00 keys list
q:
	cored-00 q bank balances $(_WALLET_ADDRESS_)
fund:
	cored-00 tx bank send alice $(_WALLET_ADDRESS_) 10000000udevcore --broadcast-mode=block

instantiate:
	cored-00 tx wasm instantiate $(CODE_ID) \
	"{\"members\":[\"devcore17vlkqgpxnrqqrgq877v7a4ymfma0r03g02agg8\"]}" \
	--amount="10000000$(COREUM_DENOM)" --no-admin --label "dao contract" --from ${DEV_WALLET} --gas auto --gas-adjustment 1.3 -b block -y $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS)

contract_address:
	cored-00 q wasm list-contract-by-code $(CODE_ID) --output json $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS) | jq -r '.contracts[-1]'
	CONTRACT_ADDRESS=$(shell cored-00 q wasm list-contract-by-code $(CODE_ID) --output json $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS) | jq -r '.contracts[-1]')
	echo $$CONTRACT_ADDRESS
