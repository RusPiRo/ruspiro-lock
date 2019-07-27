/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/

//! # Semaphore implementation. It uses atomic memory locks to ensure the semaphore is exclusively accessed while checking
//! or updating it's value. On Raspbarry Pi this will only work if the MMU has been properly configured. Otherwise those
//! operations may just hang.
//! 

use core::sync::atomic::{AtomicBool, Ordering, fence};

pub struct Semaphore {
    flag: AtomicBool,
    count: u32
}

impl Semaphore {
    /// Instantiate a new semaphore with a given initial value
    /// # Example
    /// ```
    /// let mut sema = Semaphore::new(5); // semaphore could be used/aquired 5 times
    /// ```
    pub const fn new(initial: u32) -> Semaphore {
        Semaphore {
            flag: AtomicBool::new(false),
            count: initial,
        }
    }

    /// increase the inner count of a semaphore allowing it to be used as many times as the inner counters value
    /// 
    /// # Example
    /// ```
    /// let mut sema = Semaphore::new(0);
    /// 
    /// sema.up(); // the counter of the semaphore will be increased
    /// ```
    pub fn up(&mut self) {
        // ensure atomic access to the count value so it is not updated from other cores while updating
        while self.flag.compare_and_swap(false, true, Ordering::Relaxed) != false { }
        fence(Ordering::Acquire);
        self.count += 1;
        // release the atomic access
        self.flag.store(false, Ordering::Release);
    }

    /// decrease the inner count of a semaphore. This blocks the current core if the current count is 0
    /// and could not beeing decreased. For an unblocking operation use [try_down]
    /// 
    /// # Example
    /// ```
    /// let mut sema = Semaphore::new(0);
    /// 
    /// sema.down();
    /// // if we reache this line, we have used the semaphore and decreased the counter by 1
    /// ```
    pub fn down(&mut self) {
        loop {
            if self.try_down().is_ok() {
                return;
            }
            // a small hack to force some cpu cycles to pass before the next try
            // may be a timer wait in the future?
            for _ in 0..1000 {
                unsafe { asm!("nop") };
            }
        }
    }

    /// try to decrease a semaphore for usage. Returns [Ok()] if the semaphore could be used.
    /// 
    /// # Example
    /// ```
    /// let mut sema = Semaphore::new(0);
    /// 
    /// if sema.try_down().is_ok() {
    ///     // do something... the counter of the semaphore has been decreased by 1
    /// }
    /// ```
    pub fn try_down(&mut self) -> Result<(),()> {
        while self.flag.compare_and_swap(false, true, Ordering::Relaxed) != false { }
        fence(Ordering::Acquire);
        // try to decrease the counter
        let success = if self.count > 0 {
            self.count -= 1;
            true
        } else {
            false
        };
        // release the atomic access
        self.flag.store(false, Ordering::Release);
        if success {
            Ok(())
        } else {
            Err(())
        }
    }
}