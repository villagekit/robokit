build:
	cargo build --release

flash:
	cargo flash --chip stm32f767zitx --release
