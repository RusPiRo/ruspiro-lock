/***************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **************************************************************************************************/

//! # Semaphore implementation.
//!
//! It uses atomic memory locks to ensure the semaphore is exclusively accessed while checking
//! or updating it's value. On Raspbarry Pi this will only work if the MMU has been properly configured. Otherwise those
//! operations may just hang.
//!
//! # Example
//! ```
//! use ruspiro_lock::Semaphore;
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
use core::sync::atomic::{AtomicBool, AtomicU16, Ordering};

/// Simple counting blocking or non-blocking lock
#[derive(Debug)]
#[repr(C, align(16))]
pub struct Semaphore {
    flag: AtomicBool,
    count: AtomicU16, //Cell<u32>,
}

impl Semaphore {
    /// Instantiate a new semaphore with a given initial value
    /// # Example
    /// ```
    /// # use ruspiro_lock::Semaphore;
    /// # fn doc() {
    ///     let mut sema = Semaphore::new(5); // semaphore could be used/aquired 5 times
    /// # }
    /// ```
    pub const fn new(initial: u16) -> Semaphore {
        Semaphore {
            flag: AtomicBool::new(false),
            count: AtomicU16::new(initial), //Cell::new(initial),
        }
    }

    /// increase the inner count of a semaphore allowing it to be used as many times as the inner counters value
    ///
    /// # Example
    /// ```no_run
    /// # use ruspiro_lock::Semaphore;
    /// # fn doc() {
    ///     let mut sema = Semaphore::new(0);
    ///     sema.up(); // the counter of the semaphore will be increased
    /// # }
    /// ```
    pub fn up(&self) {
        while self.flag.compare_and_swap(false, true, Ordering::SeqCst) {}
        self.count.fetch_add(1, Ordering::AcqRel);
        self.flag.store(false, Ordering::Release);
    }

    /// decrease the inner count of a semaphore. This blocks the current core if the current count is 0
    /// and could not beeing decreased. For an unblocking operation use [Semaphore::try_down]
    ///
    /// # Example
    /// ```no_run
    /// # use ruspiro_lock::Semaphore;
    /// # fn doc() {
    ///     let sema = Semaphore::new(0);
    ///     sema.down();
    ///     // if we reache this line, we have used the semaphore and decreased the counter by 1
    /// # }
    /// ```
    pub fn down(&self) {
        loop {
            if self.try_down().is_ok() {
                return;
            }
        }
    }

    /// try to decrease a semaphore for usage. Returns [Ok()] if the semaphore could be used.
    ///
    /// # Example
    /// ```
    /// # use ruspiro_lock::Semaphore;
    /// # fn doc() {
    ///     let sema = Semaphore::new(0);
    ///     if sema.try_down().is_ok() {
    ///         // do something... the counter of the semaphore has been decreased by 1
    ///     }
    /// # }
    /// ```
    pub fn try_down(&self) -> Result<(), ()> {
        // we need to deactivate interrupts as this wait should never beeing interrupted
        // otherwise it could lead to deadlocks
        while self.flag.compare_and_swap(false, true, Ordering::SeqCst) {}
        if self.count.load(Ordering::Acquire) > 0 {
            self.count.fetch_sub(1, Ordering::AcqRel);
            self.flag.store(false, Ordering::Release);
            Ok(())
        } else {
            self.flag.store(false, Ordering::Release);
            Err(())
        }
    }
}

unsafe impl Sync for Semaphore {}
unsafe impl Send for Semaphore {}
