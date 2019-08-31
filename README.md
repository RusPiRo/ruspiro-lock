# RusPiRo Lock crate

Simple to use abstractions on low level atomic locks:
 - ``Spinlock``: blocking lock to secure cross core mutual exclusive access (requires a configured MMU on Raspberry Pi)
 - ``Semaphore``: counting blocking or non-blocking lock to secure cross core exclusive access
 - ``DataLock``: data container guarded by a non-blocking atomic lock to secure cross core mutual exclusive access

[![Travis-CI Status](https://api.travis-ci.org/RusPiRo/ruspiro-lock.svg?branch=master)](https://travis-ci.org/RusPiRo/ruspiro-lock)
[![Latest Version](https://img.shields.io/crates/v/ruspiro-lock.svg)](https://crates.io/crates/ruspiro-lock)
[![Documentation](https://docs.rs/ruspiro-lock/badge.svg)](https://docs.rs/ruspiro-lock)
[![License](https://img.shields.io/crates/l/ruspiro-lock.svg)](https://github.com/RusPiRo/ruspiro-lock#license)

# Usage

To use this crate simply add the dependency to your ``Cargo.toml`` file:
```
[dependencies]
ruspiro-lock = "0.2"
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

Using data container with atmic lock guard:
```
use ruspiro_lock::*;

static DATA: DataLock<u32> = DataLock::new(0);

fn main() {
    if let Some(mut data) = DATA.try_lock() {
        *data = 20;
    }
    // once the data goes ot of scope the lock will be released
    if let Some(data) = DATA.try_lock() {
        println!("data: {}", *data);
    
        // another lock should fail inside this scope
        assert_eq!(DATA.try_lock(), None);
    }
}
```


## License
Licensed under Apache License, Version 2.0, ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)