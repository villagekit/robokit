# gridbot-software

Micro-controller: [Nucleo-144 STM32-F767ZI](https://nz.element14.com/stmicroelectronics/nucleo-f767zi/dev-board-nucleo-32-mcu/dp/2546569)

## Development

### Setup

#### Rust

With [`rustup`](https://rustup.rs) installed, install the toolchain:

( https://docs.rust-embedded.org/cortex-m-quickstart/cortex_m_quickstart/ )

```shell
rustup target add thumbv7em-none-eabi
```

#### Binary utils

```shell
sudo apt install build-essential
```

```shell
cargo install cargo-binutils
```

```shell
rustup component add llvm-tools-preview
```

> Alternatively, install GNU Binutils for ARM:
>
> ```
> sudo apt install binutils-arm-none-eabi
> ```

#### Flash tools

```shell
sudo apt install pkg-config libusb-1.0-0-dev libudev-dev
```

```shell
cargo install cargo-flash
```

> Alternatively, to flash onto the STM32, install [`stlink-tools`](https://github.com/stlink-org/stlink)
>
> ```shell
> sudo apt install stlink-tools
> ```

### Dev tools

```shell
cargo install flip-link
cargo install probe-run
```

### Run

Run:

```shell
cargo run
```

### Build and Flash

Build:

```shell
cargo build --release
```

Flash:

```shell
cargo flash --chip stm32f767zitx --release
```

### Test

Unit tests:

```shell
cargo test --lib
```

Integration tests:

```shell
cargo test --test integration
```
