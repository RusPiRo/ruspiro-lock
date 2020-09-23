/***********************************************************************************************************************
 * Copyright (c) 2020 by the authors
 *
 * Author: André Borrmann <pspwizard@gmx.de>
 * License: Apache License 2.0 / MIT
 **********************************************************************************************************************/

//! # Mutex
//!
//! Enable exclusive access to data guarded by a cross core atomic lock. In contrast to a ``Singleton``
//! the data access lock could also be non-blocking and might fail. But exclusive access is guaranteed
//! across cores if the lock could be aquired.
//!
//! # Example
//! ```
//! use ruspiro_lock::Mutex;
//!
//! static DATA: Mutex<u32> = Mutex::new(0);
//!
//! fn main() {
//!     if let Some(mut data) = DATA.try_lock() {
//!         *data = 20;
//!     }
//!     // once the data goes ot of scope the lock will be released
//!     if let Some(data) = DATA.try_lock() {
//!         println!("data: {}", *data);
//!
//!         // another lock should fail inside this scope
//!         assert!(DATA.try_lock().is_none());
//!     }
//! }
//! ```
//! This example uses a ``static`` variable to define a lock that shall be available across cores.
//! The data might also be wrapped in an ``Arc<Mutex<T>>`` and shared between cores using clones
//! of the ``Arc``.
//!

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// An mutual exclusive access lock for the interior data
#[repr(C, align(16))]
pub struct Mutex<T: ?Sized> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

/// The MutexGuard is the result of successfully aquiring the mutual exclusive lock for the interior
/// data. If this guard goes ot of scope the lock will be released
#[derive(Debug)]
pub struct MutexGuard<'a, T> {
    _data: &'a Mutex<T>,
}

impl<T> Mutex<T> {
    /// Create a new data access guarding lock
    pub const fn new(value: T) -> Self {
        Mutex {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    /// Try to lock the interior data for mutual exclusive access. Returns ``None`` if the lock failes
    /// or ``Some(MutexGuard)``. The actual data, the MutexGuard wraps could be conviniently accessed by
    /// dereferencing it.
    ///
    /// # Example
    /// ```
    /// # use ruspiro_lock::Mutex;
    /// static DATA: Mutex<u32> = Mutex::new(10);
    /// # fn main() {
    ///     if let Some(data) = DATA.try_lock() {
    ///         // do something with data
    ///     }
    /// # }
    /// ```
    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        // do the atomic operation to set the lock
        if !self.locked.swap(true, Ordering::Acquire) {
            // has been false previously means we now have the lock

            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            unsafe {
                // dmb required before allow access to the protected resource, see:
                // http://infocenter.arm.com/help/topic/com.arm.doc.dht0008a/DHT0008A_arm_synchronization_primitives.pdf
                llvm_asm!("dmb sy");
            }

            Some(MutexGuard { _data: self })
        } else {
            // we couldn't set the lock
            None
        }
    }

    /// Lock the guarded data for mutual exclusive access. This blocks until the data could be
    /// successfully locked. The locked data will be returned as ``MutexGuard``. Simply dereferencing
    /// this allows access to the contained data value.
    ///
    /// # Example
    /// ```
    /// # use ruspiro_lock::Mutex;
    /// static DATA: Mutex<u32> = Mutex::new(10);
    /// # fn main() {
    ///     let mut data = DATA.lock();
    ///     // do something with data
    ///     *data = 15;
    ///
    /// # }
    /// ```
    pub fn lock(&self) -> MutexGuard<T> {
        loop {
            if let Some(data) = self.try_lock() {
                return data;
            }
            // to save energy and cpu consumption we can wait for an event beeing raised that indicates that the
            // mutex lock have liekly been released
            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            unsafe {
                llvm_asm!("wfe");
            }
        }
    }
}

impl<T> core::fmt::Debug for Mutex<T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Mutex")
            .field("Value", unsafe { &*self.data.get() })
            .finish()
    }
}

// when the MutexGuard is dropped release the owning lock
impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self._data.locked.swap(false, Ordering::Release);

        #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
        unsafe {
            // dmb required before allow access to the protected resource, see:
            // http://infocenter.arm.com/help/topic/com.arm.doc.dht0008a/DHT0008A_arm_synchronization_primitives.pdf
            llvm_asm!("dmb sy");
            // also raise a signal to indicate the mutex has been changed (this trigger all WFE's to continue
            // processing) but do data syncronisation barrier upfront to ensure any data updates has been finished
            llvm_asm!(
                "dsb sy
                 sev"
            );
        }
    }
}

// Dereferencing the value contained in the [MutexGuard]
// this ok as the MutexGuard does only exist if the exclusive access to the data could
// be ensured. Therefore also only one `MutexGuard` could ever exist for one specific ``Mutex``, which makes it
// safe to return immutable and mutable references.
impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self._data.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self._data.data.get() }
    }
}

/// The Mutex is always `Sync`, to make it `Send` as well it need to be wrapped into an `Arc`.
unsafe impl<T> Sync for Mutex<T> {}
