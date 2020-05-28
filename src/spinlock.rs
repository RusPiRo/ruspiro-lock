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

/// A blocking cross core lock to guarantee mutual exclusive access. While this lock might block other cores
/// to continue processing this lock should be held as short as possible. Also care shall be taken
/// while using this lock within interrupt handlers, as this might lead to deadlock situations if the
/// lock holding core is interrupted and the interrupt is also trying to aquire the same lock.
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
    #[inline]
    pub fn aquire(&self) {
        // set the atomic value to true if it has been false before (set the lock)
        while self.flag.compare_and_swap(false, true, Ordering::SeqCst) {}

        #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
        unsafe {
            // dmb required before allow access to the protected resource, see:
            // http://infocenter.arm.com/help/topic/com.arm.doc.dht0008a/DHT0008A_arm_synchronization_primitives.pdf
            llvm_asm!("dmb sy");
        }
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
    #[inline]
    pub fn release(&self) {
        self.flag.store(false, Ordering::SeqCst);

        #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
        unsafe {
            // dmb required before allow access to the protected resource, see:
            // http://infocenter.arm.com/help/topic/com.arm.doc.dht0008a/DHT0008A_arm_synchronization_primitives.pdf
            llvm_asm!("dmb sy");
            // also raise a signal to indicate the spinlock has been changed (this trigger all WFE's to continue 
            // processing) but do data syncronisation barrier upfront to ensure any data updates has been finished
            llvm_asm!(
                "dsb sy
                 sev"
            );
        }
    }
}
