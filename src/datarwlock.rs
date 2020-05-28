/***************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **************************************************************************************************/

//! # Data Read/Write Lock
//!
//! Enable exclusive write and shared read access to data guarded by a cross core atomic lock.
//! In contrast to a ``Singleton`` the data write access lock could also be non-blocking and might fail. But exclusive 
//! access is guaranteed across cores if the lock could be aquired. Shared read access is always possible to such a 
//! resource.
//!
//! # Example
//! ```
//! use ruspiro_lock::DataRWLock;
//!
//! static DATA: DataRWLock<u32> = DataRWLock::new(0);
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
//! The data might also be wrapped in an ``Arc<DataRWLock<T>>`` and shared between cores using clones
//! of the ``Arc``.
//!

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// An exclusive access lock around the given data
#[repr(C, align(16))]
pub struct DataRWLock<T> {
    /// indicates whether a mutual exclusive write lock exists
    write_lock: AtomicBool,
    /// counts existing read-locks, this could be used in future to mark the data as "dirty" if a write lock is aquired
    /// whiled read access is also handed out. Should a write access request fail with existing read access ?
    read_locks: AtomicU32,
    data: UnsafeCell<T>,
}

/// Result of trying to access the data using ``try_lock`` or ``lock`` on the data lock. If the
/// result goes out of scope the write lock is released.
#[derive(Debug)]
pub struct DataWriteLock<'a, T> {
    _data: &'a DataRWLock<T>,
}

/// Result of aquiring read access to the data using ``read`` on the data lock. If the
/// result goes out of scope the read lock is released.
#[derive(Debug)]
pub struct DataReadLock<'a, T> {
    _data: &'a DataRWLock<T>,
}

impl<T> DataRWLock<T> {
    /// Create a new data access guarding lock.
    pub const fn new(value: T) -> Self {
        DataRWLock {
            write_lock: AtomicBool::new(false),
            read_locks: AtomicU32::new(0),
            data: UnsafeCell::new(value),
        }
    }

    /// Try to lock the guarded data for mutual exclusive access. Returns ``None`` if the lock fails
    /// or ``Some(DataWriteLock)``. The actual data, the [DataWriteLock] wraps could be conviniently accessed by
    /// dereferencing it.
    ///
    /// # Example
    /// ```
    /// # use ruspiro_lock::DataRWLock;
    /// static DATA: DataRWLock<u32> = DataRWLock::new(10);
    /// # fn main() {
    ///     if let Some(data) = DATA.try_lock() {
    ///         // do something with data
    ///     }
    /// # }
    /// ```
    pub fn try_lock(&self) -> Option<DataWriteLock<T>> {
        // do the atomic operation to set the lock
        if !self.write_lock.swap(true, Ordering::SeqCst) {
            // has been false previously means we now have the lock

            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            unsafe {
                // dmb required before allow access to the protected resource, see:
                // http://infocenter.arm.com/help/topic/com.arm.doc.dht0008a/DHT0008A_arm_synchronization_primitives.pdf
                llvm_asm!("dmb sy");
            }

            Some(DataWriteLock { _data: self })
        } else {
            // we couldn't set the lock
            None
        }
    }

    /// Lock the guarded data for mutual exclusive access. This blocks until the data could be
    /// successfully locked. The locked data will be returned as ``TryDataLock``. Simply derefrencing
    /// this allows access to the contained data value.
    ///
    /// # Example
    /// ```
    /// # use ruspiro_lock::DataRWLock;
    /// static DATA: DataRWLock<u32> = DataRWLock::new(10);
    /// # fn main() {
    ///     let mut data = DATA.lock();
    ///     // do something with data
    ///     *data = 15;
    ///
    /// # }
    /// ```
    pub fn lock(&self) -> DataWriteLock<T> {
        loop {
            if let Some(data) = self.try_lock() {
                return data;
            }
            // to save energy and cpu consumption we can wait for an event beeing raised that indicates that the 
            // semaphore value has likely beeing changed
            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            unsafe { llvm_asm!("wfe"); }
        }
    }

    pub fn read(&self) -> DataReadLock<T> {
        self.read_locks.fetch_add(1, Ordering::AcqRel);
        DataReadLock { _data: self }
    }
}

impl<T> core::fmt::Debug for DataRWLock<T> 
    where T: core::fmt::Debug
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DataLock")
            .field("Value", unsafe { &*self.data.get() })
            .field("ReadLocks", &self.read_locks)
            .finish()
    }
}

// when the DataWriteLock is dropped release the owning lock
impl<T> Drop for DataWriteLock<'_, T> {
    fn drop(&mut self) {
        self._data.write_lock.swap(false, Ordering::SeqCst);

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
}

// when the DataWriteLock is dropped release the owning lock
impl<T> Drop for DataReadLock<'_, T> {
    fn drop(&mut self) {
        self._data.read_locks.fetch_sub(1, Ordering::AcqRel);

        #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
        unsafe {
            // dmb required after atomic operations, see:
            // http://infocenter.arm.com/help/topic/com.arm.doc.dht0008a/DHT0008A_arm_synchronization_primitives.pdf
            llvm_asm!("dmb sy");
        }
    }
}

// dereferencing the value contained in the DataWriteLock
// this is ok as the DataWriteLock does only exist if the exclusive access to the data could
// be ensured. Therefore also only one ``DataWriteLock`` could ever exist for one specific ``DataRWLock``, which makes 
// it safe to return immutable and mutable references.
impl<T> Deref for DataWriteLock<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self._data.data.get() }
    }
}

impl<T> DerefMut for DataWriteLock<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self._data.data.get() }
    }
}

// the ``DataReadLock`` can only be immutable dereferenced
impl<T> Deref for DataReadLock<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self._data.data.get() }
    }
}

unsafe impl<T> Sync for DataRWLock<T> {}
unsafe impl<T> Send for DataRWLock<T> {}
