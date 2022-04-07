run:
	cargo run

test:
	cargo test --lib

test-e2e:
	cargo test --test integration

build:
	cargo build --release

flash:
	cargo flash --chip STM32F767ZITx --release

flash-reset:
	cargo flash --chip STM32F767ZITx --release --connect-under-reset
