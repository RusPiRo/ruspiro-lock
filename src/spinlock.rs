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
//! use ruspiro_lock::Spinlock;
//! 
//! static LOCK: Spinlock = Spinlock::new();
//! 
//! fn main () {
//!     LOCK.aquire(); // will only return if the lock could be set
//!     // do something
//! 
//!     LOCK.release(); // releasing the lock
//! }
//! ```
use core::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
#[repr(C, align(16))]
pub struct Spinlock {
    flag: AtomicBool,
}

impl Spinlock {
    /// Create a new Spinlock. To ensure it is shared between cores, it's typically assigned to a static variable
    /// # Example
    /// ```
    /// # use ruspiro_lock::Spinlock;
    /// static LOCK: Spinlock = Spinlock::new();
    /// ```
    pub const fn new() -> Spinlock {
        Spinlock {
            flag: AtomicBool::new(false),
        }
    }

    /// Aquire a spinlock. This will block the current core until the lock could be aquired.
    /// # Example
    /// ```no_run
    /// # use ruspiro_lock::Spinlock;
    /// static LOCK: Spinlock = Spinlock::new();
    /// # fn main() {
    ///     LOCK.aquire();
    ///     // execution continues only if the lock could be aquired
    /// # }
    /// ```
    pub fn aquire(&self) {
        // set the atomic value to true if it has been false before (set the lock)
        // we need to deactivate interrupts as this wait and the aquired lock should never beeing interrupted
        // otherwise it could lead to deadlocks
        crate::disable_interrupts();
        while self.flag.compare_and_swap(false, true, Ordering::SeqCst) != false { }
    }

    /// Release an aquired spinlock.
    /// # Example
    /// ```no_run
    /// # use ruspiro_lock::Spinlock;
    /// static LOCK: Spinlock = Spinlock::new();
    /// # fn main() {
    ///     LOCK.release();
    /// # }
    /// ```
    pub fn release(&self) {
        self.flag.store(false, Ordering::SeqCst);
        // re-activate interrupts to the previous enable-state as the lock is now released
        // and no interrupt deadlocks can occur
        crate::re_enable_interrupts();
    }
}
