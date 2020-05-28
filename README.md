# Shared I2C in RTIC - exercise

Basically, this solution consists of 2 major elements.

- `I2cHandlerProxy` which pretends to be the embedded_hal's i2c implementation.
  It is supposed to be consumed by a driver object during init phase.
  - It spawns i2c-related task via RTIC's `Spawn` object.

- `ScopedTaskSpawnProvider` which creates something similar to scope.
  - On creation, it populates `I2cHandlerProxy` with `Spawn` instance.
  - On drop, it sets `I2cHandlerProxy` `Spawn` reference to `None`.
    - After drop, accessing i2c via `I2cHandlerProxy` causes panic.

I reworked it and now it is runnable in `qemu-system-arm` with `cargo run`.

## Major soundness issues
- `STSP` accepts any `Spawn` object (actually any object that implements `I2cHandlerCallable`)
  - Therefore, it will be transmuted into any other `I2cHandlerCallable` implementing type causing UB.
  - It is a result of struggles with lifetimes. More information in the code and comments.
- `I2cCommand` holds references with `static` lifetimes. Passing it to any spawned task with
  lower priority than i2c handler is a potential `use-after-free`.