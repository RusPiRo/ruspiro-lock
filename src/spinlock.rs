/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/

//! # Spinlock
//! Providing simple atomic Spinlock. This can be used to ensure cross core atomic access to data that is typically
//! shared between them. For example MMIO mapped registers that allow access to peripherals. Please note that usage
//! of Spinlocks on Raspberry Pi is only safe if the MMU has ben configured properly. Otherwise the cores trying to aquire
//! a lock will just hang, even if the lock would be available to them.
//! 
//! # Example
//! ```
//! static LOCK: Spinlock = Spinlock::new();
//! 
//! fn main () {
//!     LOCK.aquire(); // will only return if the lock could be set
//!     // do something
//! 
//!     LOCK.release(); // releasing the lock
//! }
//! ```
use core::sync::atomic::{AtomicBool, Ordering, fence};

pub struct Spinlock {
    flag: AtomicBool,
}

impl Spinlock {
    /// Create a new Spinlock. To ensure it is shared between cores, it's typically assigned to a static variable
    /// # Example
    /// ```
    /// static LOCK: Spinlock = Spinlock::new();
    /// ```
    pub const fn new() -> Spinlock {
        Spinlock {
            flag: AtomicBool::new(false),
        }
    }

    pub fn aquire(&self) {
        // set the atomic value to true if it has been false before
        // it returns "false" (the old value) if the value could be changed to true
        while self.flag.compare_and_swap(false, true, Ordering::Relaxed) != false { }
        // the fence ensures propper ordering with the unlock call
        fence(Ordering::Acquire);
    }

    pub fn release(&self) {
        self.flag.store(false, Ordering::Release);
    }
}