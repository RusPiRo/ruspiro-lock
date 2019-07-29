/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-lock/0.0.2")]
#![no_std]
#![feature(asm)]

//! # Atomic locks for Raspberry Pi baremetal systems
//! This crate provides two options of locks. [Spinlock] and [Semaphore]. They provide mechanisms to secure cross core
//! access to shared data like MMIO registers of peripherals. As the locks depend on low level atomics they do only work
//! on the Raspberry Pi if the MMU is properly configured. Otherwise using either of the lock functions will hang the 
//! core it has been used on.
//! 
//! # Usage
//! Using a Spinlock to ensure exclusive access.
//! ```
//! use ruspiro_lock::*;
//! 
//! static SPIN: Spinlock = Spinlock::new();
//! 
//! fn main() {
//!     SPIN.aquire();
//!     // following code is only executed if the lock could be aquired, the executing core pause till then
//!     let _ = 10 + 3;
//!     SPIN.release();
//! }
//! ```
//! 
//! using a Semaphore to specify how often specific access is valid
//! 
//! ```
//! use ruspriro_lock::*;
//! 
//! static mut SEMA: Semaphore = Semaphore::new(1);
//! 
//! fn main() {
//!     unsafe { // unsafe necessary as accessing mut static's is unsafe
//!         if SEMA.try_down().is_ok() {
//!             // we gained access to the semaphore, do something
//!             let _ = 20 /4;
//!             SEMA.up();
//!         }
//!     }
//! }
//! ```
//! 

pub mod spinlock;
pub use self::spinlock::*;

pub mod semaphore;
pub use self::semaphore::*;