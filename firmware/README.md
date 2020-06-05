# Seriallogger Firmware


## Build and flash

Use flash storage:

```bash
bobbin load --bin seriallogger --features use_flash
```

Erase flash storage

```bash
bobbin load --bin seriallogger --features full_erase
```

No features

```bash
bobbin load --bin seriallogger

# or

Cargo run

# or

Cargo build
```