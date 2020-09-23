/***********************************************************************************************************************
 * Copyright (c) 2020 by the authors
 *
 * Author: André Borrmann <pspwizard@gmx.de>
 * License: Apache License 2.0 / MIT
 **********************************************************************************************************************/

//! # Semaphore implementation.
//!
//! It uses atomic memory locks to ensure the semaphore is exclusively accessed while checking
//! or updating it's value. On Raspbarry Pi this will only work if the MMU has been properly configured. Otherwise those
//! operations may just hang.
//!
//! # Example
//! ```
//! use ruspiro_lock::sync::Semaphore;
//!
//! static SEMA: Semaphore = Semaphore::new(1);
//!
//! fn main () {
//!     SEMA.down(); // will only return if the counter could be decreased
//!     // do something
//!
//!     SEMA.up(); // increase the counter for another usage
//! }
//! ```
use core::sync::atomic::{AtomicU32, Ordering};

/// Simple counting blocking or non-blocking lock
#[derive(Debug)]
#[repr(C, align(16))]
pub struct Semaphore {
    count: AtomicU32,
}

impl Semaphore {
    /// Instantiate a new semaphore with a given initial value
    /// # Example
    /// ```
    /// # use ruspiro_lock::sync::Semaphore;
    /// # fn doc() {
    ///     let mut sema = Semaphore::new(5); // semaphore could be used/aquired 5 times
    /// # }
    /// ```
    pub const fn new(initial: u32) -> Semaphore {
        Semaphore {
            count: AtomicU32::new(initial),
        }
    }

    /// increase the inner count of a semaphore allowing it to be used as many times as the inner counters value
    ///
    /// # Example
    /// ```no_run
    /// # use ruspiro_lock::sync::Semaphore;
    /// # fn doc() {
    ///     let mut sema = Semaphore::new(0);
    ///     sema.up(); // the counter of the semaphore will be increased
    /// # }
    /// ```
    #[inline]
    pub fn up(&self) {
        self.count.fetch_add(1, Ordering::AcqRel);

        #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
        unsafe {
            // dmb required before allow access to the protected resource, see:
            // http://infocenter.arm.com/help/topic/com.arm.doc.dht0008a/DHT0008A_arm_synchronization_primitives.pdf
            llvm_asm!("dmb sy");
            // also raise a signal to indicate the semaphore has been changed (this trigger all WFE's to continue
            // processing) but do data syncronisation barrier upfront to ensure any data updates has been finished
            llvm_asm!(
                "dsb sy
                 sev"
            );
        }
    }

    /// decrease the inner count of a semaphore. This blocks the current core if the current count is 0
    /// and could not beeing decreased. For an unblocking operation use [Semaphore::try_down]
    ///
    /// # Example
    /// ```no_run
    /// # use ruspiro_lock::sync::Semaphore;
    /// # fn doc() {
    ///     let sema = Semaphore::new(0);
    ///     sema.down();
    ///     // if we reache this line, we have used the semaphore and decreased the counter by 1
    /// # }
    /// ```
    #[inline]
    pub fn down(&self) {
        loop {
            if self.try_down().is_ok() {
                return;
            }
            // to save energy and cpu consumption we can wait for an event beeing raised that indicates that the
            // semaphore value has likely beeing changed
            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            unsafe {
                llvm_asm!("wfe");
            }
        }
    }

    /// try to decrease a semaphore for usage. Returns [Ok()] if the semaphore could be used.
    ///
    /// # Example
    /// ```
    /// # use ruspiro_lock::sync::Semaphore;
    /// # fn doc() {
    ///     let sema = Semaphore::new(0);
    ///     if sema.try_down().is_ok() {
    ///         // do something... the counter of the semaphore has been decreased by 1
    ///     }
    /// # }
    /// ```
    #[inline]
    pub fn try_down(&self) -> Result<(), ()> {
        let mut value = self.count.load(Ordering::Acquire);
        if value > 0 {
            value -= 1;
            self.count.store(value, Ordering::Release);
            // dmb required before allow access to the protected resource see:
            // http://infocenter.arm.com/help/topic/com.arm.doc.dht0008a/DHT0008A_arm_synchronization_primitives.pdf
            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            unsafe {
                llvm_asm!("dmb sy");
            }
            Ok(())
        } else {
            // set the current value as "dummy" store to clear the atomic monitor
            self.count.store(value, Ordering::Release);
            Err(())
        }
    }
}

impl Default for Semaphore {
    fn default() -> Self {
        Semaphore::new(0)
    }
}

unsafe impl Sync for Semaphore {}
unsafe impl Send for Semaphore {}
