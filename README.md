# RusPiRo Lock crate

This crates provide two simple abstractions on low level atomic locks. The ``Spinlock`` and the ``Semaphore``.

[![Travis-CI Status](https://api.travis-ci.org/RusPiRo/ruspiro-lock.svg?branch=master)](https://travis-ci.org/RusPiRo/ruspiro-lock)
[![Latest Version](https://img.shields.io/crates/v/ruspiro-lock.svg)](https://crates.io/crates/ruspiro-lock)
[![Documentation](https://docs.rs/ruspiro-lock/badge.svg)](https://docs.rs/ruspiro-lock)
[![License](https://img.shields.io/crates/l/ruspiro-lock.svg)](https://github.com/RusPiRo/ruspiro-lock#license)

# Usage

To use this crate simply add the dependency to your ``Cargo.toml`` file:
```
[dependencies]
ruspiro-lock = "0.1.0"
```

Once done the definition and usage of the locks is as follows:

Using a Spinlock to ensure exclusive access:
```
use ruspiro_lock::*;

static SPIN: Spinlock = Spinlock::new();

fn main() {
    SPIN.aquire();
    // following code is only executed if the lock could be aquired, the executing core pause till then
    let _ = 10 + 3;
    SPIN.release();
}
```

Using a Semaphore to specify how often specific access is valid:
```
use ruspriro_lock::*;

static mut SEMA: Semaphore = Semaphore::new(1);

fn main() {
    unsafe { // unsafe necessary as accessing mut static's is unsafe
        if SEMA.try_down().is_ok() {
            // we gained access to the semaphore, do something
            let _ = 20 /4;
            SEMA.up();
        }
    }
}
```

## License
Licensed under Apache License, Version 2.0, ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)