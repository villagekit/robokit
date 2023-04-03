run:
	cargo run

test-robokit:
	cargo test --lib --package robokit --target x86_64-unknown-linux-gnu

test-gridbot:
	cargo test --lib --package gridbot-tahi

test-e2e:
	cargo test --test integration

build:
	cargo build --release

flash:
	cargo flash --chip STM32F767ZITx --release

flash-reset:
	cargo flash --chip STM32F767ZITx --release --connect-under-reset
