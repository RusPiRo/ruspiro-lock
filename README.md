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
ruspiro-lock = "0.3"
```

Once done the definition and usage of the locks is as follows. Keep in mind to share those locking
primitives accross the Rasperry Pi cores they should be wrapped in an ``Arc``.

## Spinlock
```
use ruspiro_lock::Spinlock;

fn main() {
    let spin = Spinlock::new();
    spin.aquire();
    // following code is only executed if the lock could be aquired, the executing core pause till then
    let _ = 10 + 3;
    spin.release();
}
```

## Semaphore
```
use ruspiro_lock::Semaphore;

fn main() {
    let sema  = Semaphore::new(1);
    if sema.try_down().is_ok() {
        // we gained access to the semaphore, do something
        let _ = 20 /4;
        sema.up();
    }
}
```

## DataLock
```
use ruspiro_lock::DataLock;

fn main() {
    let dalo = DataLock::new(0u32);
    if let Some(mut data) = dalo.try_lock() {
        *data = 20;
    }
    // once the data goes ot of scope the lock will be released
    if let Some(data) = dalo.try_lock() {
        println!("data: {}", *data);
    
        // another lock should fail inside this scope
        assert!(dalo.try_lock().is_none());
    }
    
    // a blocking lock on the data will block the current execution until the lock get's available
    let mut data = dalo.lock();
    *data = 12;
}
```

## License
Licensed under Apache License, Version 2.0, ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)