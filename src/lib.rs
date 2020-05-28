/***************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-lock/0.3.3")]
#![no_std]
#![feature(llvm_asm)]

//! # Atomic locks for Raspberry Pi baremetal systems
//!
//! This crate provides two options of locks and a data access guard. [Spinlock], [Semaphore], [DataLock].
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
//! use ruspiro_lock::DataLock;
//!
//! fn main() {
//!     let dalo = DataLock::new(0u32);
//!     if let Some(mut data) = dalo.try_lock() {
//!         *data = 20;
//!     }
//!     // once the data goes ot of scope the lock will be released
//!     if let Some(data) = dalo.try_lock() {
//!         println!("data: {}", *data);
//!     
//!         // another lock should fail inside this scope
//!         assert!(dalo.try_lock().is_none());
//!     }
//!     
//!     // a blocking lock on the data will block the current execution until the lock get's available
//!     let mut data = dalo.lock();
//!     *data = 12;
//! }
//! ```
//!

// re-export the spinlock
mod spinlock;
pub use spinlock::*;

// re-export the semaphore
mod semaphore;
pub use semaphore::*;

// re-export the data-lock
mod datalock;
pub use datalock::*;

// re-export the data read/write lock
mod datarwlock;
pub use datarwlock::*;
