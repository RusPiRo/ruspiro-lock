/***********************************************************************************************************************
 * Copyright (c) 2020 by the authors
 *
 * Author: André Borrmann <pspwizard@gmx.de>
 * License: Apache License 2.0 / MIT
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-lock/||VERSION||")]
#![cfg_attr(not(any(test, doctest)), no_std)]
#![feature(llvm_asm)]

//! # Atomic locks for Raspberry Pi baremetal systems
//!
//! This crate provides two options of locks and a data access guard. [Spinlock], [Semaphore], [Mutex].
//! They provide mechanisms to secure cross core access to shared data like MMIO registers of peripherals. As the locks
//! depend on low level atomics they do only work on the Raspberry Pi if the MMU is properly configured.
//! Otherwise using either of the lock functions will hang the core it has been used on.
//!
//! To share those locking primitives accross the Rasperry Pi cores they should be wrapped in an ``Arc``.
//!
//! # Usage
//!
//! ## Spinlock
//! ```
//! use ruspiro_lock::Spinlock;
//!
//! fn main() {
//!     let spin = Spinlock::new();
//!     spin.aquire();
//!     // following code is only executed if the lock could be aquired, the executing core pause till then
//!     let _ = 10 + 3;
//!     spin.release();
//! }
//! ```
//!
//! ## Semaphore
//! ```
//! use ruspiro_lock::Semaphore;
//!
//! fn main() {
//!     let sema  = Semaphore::new(1);
//!     if sema.try_down().is_ok() {
//!         // we gained access to the semaphore, do something
//!         let _ = 20 /4;
//!         sema.up();
//!     }
//! }
//! ```
//!
//! ## DataLock
//! ```
//! use ruspiro_lock::Mutex;
//!
//! fn main() {
//!     let mutex = Mutex::new(0u32);
//!     if let Some(mut data) = mutex.try_lock() {
//!         *data = 20;
//!     }
//!     // once the data goes ot of scope the lock will be released
//!     if let Some(data) = mutex.try_lock() {
//!         println!("data: {}", *data);
//!     
//!         // another lock should fail inside this scope
//!         assert!(mutex.try_lock().is_none());
//!     }
//!     
//!     // a blocking lock on the data will block the current execution until the lock get's available
//!     let mut data = mutex.lock();
//!     *data = 12;
//! }
//! ```

// re-export the sync lock types
pub mod sync;
// if the async feature is not enabled export the sync locks at the crates root level
#[cfg(not(fetaure = "async_locks"))]
pub use sync::*;

#[cfg(feature = "async_locks")]
pub mod r#async;
