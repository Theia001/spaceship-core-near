RFLAGS="-C link-arg=-s"

build: build-ft build-spaceship build-boxmall build-magicbox build-usn build-nft build-shippool build-riskerpool build-rankpool build-auction build-trialpool build-corepool build-collectpool build-shipmarket build-luckpool

build-auction: contracts/auction
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p auction --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/auction.wasm ./res/auction.wasm

build-boxmall: contracts/boxmall
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p boxmall --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/boxmall.wasm ./res/boxmall.wasm

build-riskerpool: contracts/riskerpool
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p riskerpool --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/riskerpool.wasm ./res/riskerpool.wasm

build-rankpool: contracts/rankpool
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p rankpool --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/rankpool.wasm ./res/rankpool.wasm

build-magicbox: contracts/magicbox
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p magicbox --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/magicbox.wasm ./res/magicbox.wasm

build-usn: contracts/mock_usn
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p mock_usn --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_usn.wasm ./res/mock_usn.wasm

build-nft: contracts/mock_nft
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p mock_nft --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_nft.wasm ./res/mock_nft.wasm

build-shippool: contracts/shippool
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p shippool --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/shippool.wasm ./res/shippool.wasm

build-ft: contracts/token-tia
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p token-tia --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/token_tia.wasm ./res/token_tia.wasm

build-spaceship: contracts/spaceship
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p spaceship --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/spaceship.wasm ./res/spaceship.wasm

build-collectpool: contracts/collectpool
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p collectpool --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/collectpool.wasm ./res/collectpool.wasm

build-corepool: contracts/corepool
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p corepool --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/corepool.wasm ./res/corepool.wasm

build-trialpool: contracts/trialpool
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p trialpool --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/trialpool.wasm ./res/trialpool.wasm

build-shipmarket: contracts/shipmarket
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p shipmarket --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/shipmarket.wasm ./res/shipmarket.wasm

build-luckpool: contracts/luckpool
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p luckpool --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/luckpool.wasm ./res/luckpool.wasm

build-mock-receiver: contracts/mock_receiver
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p mock_receiver --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_receiver.wasm ./res/mock_receiver.wasm

test: build
	RUSTFLAGS=$(RFLAGS) cargo test

test-boxmall: build
	RUSTFLAGS=$(RFLAGS) cargo test sim_buys -p boxmall -- --nocapture

test-logic: build
	RUSTFLAGS=$(RFLAGS) cargo test test_callback -p boxmall -- --nocapture

test-auction: build
	RUSTFLAGS=$(RFLAGS) cargo test test_auction -p boxmall -- --nocapture

test-shippool: build
	RUSTFLAGS=$(RFLAGS) cargo test test_shippool -p boxmall -- --nocapture

test-trialpool: build
	RUSTFLAGS=$(RFLAGS) cargo test test_trialpool -p boxmall -- --nocapture

test-corepool: build
	RUSTFLAGS=$(RFLAGS) cargo test test_corepool -p boxmall -- --nocapture

test-collectpool: build
	RUSTFLAGS=$(RFLAGS) cargo test test_collectpool -p boxmall -- --nocapture

test-shipmarket: build
	RUSTFLAGS=$(RFLAGS) cargo test test_shipmarket -p boxmall -- --nocapture

sim-auction: build-auction build-usn build-nft
	RUSTFLAGS=$(RFLAGS) cargo test -p auction -- --nocapture

sim-spaceship: build-spaceship build-mock-receiver
	RUSTFLAGS=$(RFLAGS) cargo test -p spaceship -- --nocapture

release:
	$(call docker_build,_rust_setup.sh)
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/token_tia.wasm res/token_tia_release.wasm
	cp target/wasm32-unknown-unknown/release/boxmall.wasm res/boxmall_release.wasm
	cp target/wasm32-unknown-unknown/release/riskerpool.wasm res/riskerpool_release.wasm
	cp target/wasm32-unknown-unknown/release/rankpool.wasm res/rankpool_release.wasm
	cp target/wasm32-unknown-unknown/release/magicbox.wasm res/magicbox_release.wasm
	cp target/wasm32-unknown-unknown/release/shippool.wasm res/shippool_release.wasm
	cp target/wasm32-unknown-unknown/release/spaceship.wasm res/spaceship_release.wasm
	cp target/wasm32-unknown-unknown/release/auction.wasm res/auction_release.wasm
	cp target/wasm32-unknown-unknown/release/trialpool.wasm res/trialpool_release.wasm
	cp target/wasm32-unknown-unknown/release/corepool.wasm res/corepool_release.wasm
	cp target/wasm32-unknown-unknown/release/collectpool.wasm res/collectpool_release.wasm
	cp target/wasm32-unknown-unknown/release/shipmarket.wasm res/shipmarket_release.wasm

new-release:
	$(call docker_near_build)
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/spaceship.wasm res/spaceship_release.wasm

clean:
	cargo clean
	rm -rf res/

define docker_build
	docker build -t my-contract-builder .
	docker run \
		--mount type=bind,source=${PWD},target=/host \
		--cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
		-w /host \
		-e RUSTFLAGS=$(RFLAGS) \
		-i -t my-contract-builder \
		/bin/bash $(1)
endef

define docker_near_build
	docker run \
		--rm --mount type=bind,source=${PWD},target=/host \
		--cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
		-w /host -e RUSTFLAGS=$(RFLAGS) -i -t \
		nearprotocol/contract-builder:master-83df045aded3e1b14c372ebc36a53ca71cfb4f07-amd64 \
		/bin/sh -c 'touch b.txt'
endef

# cargo build -p spaceship --target wasm32-unknown-unknown --release