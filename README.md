# RusPiRo Lock crate

API providing simple to use locks:

- `Spinlock`: blocking lock
- `Semaphore`: atomic lock counter blocking or non-blocking
- `Mutex`: blocking lock to ensure mutual exclusive to its interior.
- `RWLock`: blocking lock to provide multiple immutable and exclusive mutable access to its interior.

[![Travis-CI Status](https://api.travis-ci.com/RusPiRo/ruspiro-lock.svg?branch=master)](https://travis-ci.com/RusPiRo/ruspiro-lock)
[![Latest Version](https://img.shields.io/crates/v/ruspiro-lock.svg)](https://crates.io/crates/ruspiro-lock)
[![Documentation](https://docs.rs/ruspiro-lock/badge.svg)](https://docs.rs/ruspiro-lock)
[![License](https://img.shields.io/crates/l/ruspiro-lock.svg)](https://github.com/RusPiRo/ruspiro-lock#license)

## Usage

To use this crate simply add the dependency to your ``Cargo.toml`` file:

```toml
[dependencies]
ruspiro-lock = "0.4.0"
```

Once done the definition and usage of the locks is as follows. Keep in mind to share those locking primitives accross cores or threads they should be wrapped in an ``Arc``.

### Spinlock

```rust
use ruspiro_lock::Spinlock;

fn main() {
    let spin = Spinlock::new();
    spin.aquire();
    // following code is only executed if the lock could be aquired, the executing core pause till then
    let _ = 10 + 3;
    spin.release();
}
```

### Semaphore

```rust
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

### Mutex

```rust
use ruspiro_lock::Mutex;

fn main() {
    let mutex = Mutex::new(0u32);
    if let Some(mut data) = mutex.try_lock() {
        *data = 20;
    }
    // once the data goes ot of scope the lock will be released
    if let Some(data) = mutex.try_lock() {
        println!("data: {}", *data);

        // another lock should fail inside this scope
        assert!(mutex.try_lock().is_none());
    }

    // a blocking lock on the data will block the current execution until 
    // the lock get's available
    let mut data = mutex.lock();
    *data = 12;
}
```

### RWLock

```rust
use ruspiro_lock::RWLock;

fn main() {
    let rwlock = Arc::new(RWLock::new(0u32));
    let rwlock_clone = Arc::clone(&rwlock);
    {
        // try_lock and lock will provide a WriteLockGuard
        let mut data = rwlock.lock();
        *data = 20;
        // if a write lock exists no other write or  read lock's could be aquired
        assert!(rwlock_clone.try_lock().is_none());
        assert!(rwlock_clone.try_read().is_none());
    }
    {
        // multiple read locks are possible
        let data = rwlock.read();
        // if a write lock exists no other write or  read lock's could be aquired
        assert!(rwlock_clone.try_read().is_some());
        println!("{}", *data);
    }
}
```

## License

Licensed under Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0) or MIT ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)) at your choice.