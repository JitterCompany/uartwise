# ELW2106AA (SSD1362) OLED display proof of concept [stm32g0xx]

To retrieve stm32g0xx-hal submodule

```
git submodule init
git submodule update
```

Build the application

```
cargo build
```

# Flashing

The stm32g0 family is too new to work out of the box with OpenOCD or the Black Magic Probe. So we will use st-link.
Make sure to install the st-flash tool from `https://github.com/texane/stlink`

Build and flash

```
cargo run
```

Cargo will run `flash.sh` after building. `flash.sh` creates a binary file and uses the st-flash utility to flash the target through the st-link programmer