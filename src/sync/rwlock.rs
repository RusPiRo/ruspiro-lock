/***********************************************************************************************************************
 * Copyright (c) 2020 by the authors
 *
 * Author: André Borrmann <pspwizard@gmx.de>
 * License: Apache License 2.0 / MIT
 **********************************************************************************************************************/

//! # RWLock
//!

use core::cell::UnsafeCell;
use core::fmt;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// An exclusive access lock around the given data
#[repr(C, align(16))]
pub struct RWLock<T: ?Sized> {
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
pub struct WriteLockGuard<'a, T> {
    _data: &'a RWLock<T>,
}

/// Result of aquiring read access to the data using ``read`` on the data lock. If the
/// result goes out of scope the read lock is released.
#[derive(Debug)]
pub struct ReadLockGuard<'a, T> {
    _data: &'a RWLock<T>,
}

impl<T> RWLock<T> {
    /// Create a new data access guarding lock.
    pub const fn new(value: T) -> Self {
        RWLock {
            write_lock: AtomicBool::new(false),
            read_locks: AtomicU32::new(0),
            data: UnsafeCell::new(value),
        }
    }

    /// Try to lock the guarded data for mutual exclusive access. Returns ``None`` if the lock fails
    /// or ``Some(WriteLockGuard)``. The actual data, the [WriteLockGuard] wraps could be conviniently accessed by
    /// dereferencing it.
    pub fn try_lock(&self) -> Option<WriteLockGuard<T>> {
        if self.read_locks.load(Ordering::Relaxed) > 0 {
            // write lock can only be given if there is no concurrent ReadLock already
            // existing
            return None;
        }
        // do the atomic operation to set the lock
        if !self.write_lock.swap(true, Ordering::Acquire) {
            // has been false previously means we now have the lock

            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            unsafe {
                // dmb required before allow access to the protected resource, see:
                // http://infocenter.arm.com/help/topic/com.arm.doc.dht0008a/DHT0008A_arm_synchronization_primitives.pdf
                llvm_asm!("dmb sy");
            }

            Some(WriteLockGuard { _data: self })
        } else {
            // we couldn't set the lock
            None
        }
    }

    /// Lock the guarded data for mutual exclusive access. This blocks until the data could be
    /// successfully locked. This also implies that there is no concurrent [ReadLockGuard] existing.
    /// The locked data will be returned as [WriteLockGuard]. Simply derefrencing
    /// this allows access to the contained data value.
    ///
    pub fn lock(&self) -> WriteLockGuard<T> {
        loop {
            if let Some(write_guard) = self.try_lock() {
                //println!("write lock aquired {:?}", core::any::type_name::<T>());
                return write_guard;
            }
            // to save energy and cpu consumption we can wait for an event beeing raised that indicates that the
            // semaphore value has likely beeing changed
            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            unsafe {
                llvm_asm!("wfe");
            }
        }
    }

    /// Provide a ReadLock to the wrapped data. This call blocks until the recource is available.
    /// There can be as many concurrent [ReadLockGuard]s being handed out if there is no [WriteLockGuard] to the
    /// same resource already existing.
    pub fn try_read(&self) -> Option<ReadLockGuard<T>> {
        // read locks can only handed out if no write lock is existing already
        if self.write_lock.load(Ordering::Relaxed) {
            return None;
        } else {
            self.read_locks.fetch_add(1, Ordering::Acquire);
            //println!("read lock aquired {:?}", core::any::type_name::<T>());
            return Some(
                ReadLockGuard { _data: self }
            );
        }
    }

    /// Provide a ReadLock to the wrapped data. This call blocks until the recource is available.
    /// There can be as many concurrent [ReadLockGuard]s being handed out if there is no [WriteLockGuard] to the
    /// same resource already existing.
    pub fn read(&self) -> ReadLockGuard<T> {
        // read locks can only handed out if no write lock is existing already
        loop {
            if let Some(read_guard) = self.try_read() {
                //println!("write lock aquired {:?}", core::any::type_name::<T>());
                return read_guard;
            }

            // to save energy and cpu consumption we can wait for an event beeing raised that indicates that the
            // lock value has likely beeing changed
            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            unsafe {
                llvm_asm!("wfe");
            }
        }
    }

    /// Provide an immutable borrow to the data secured by the RWLock.
    ///
    /// # Safety
    /// This is only safe if it is guarantied that there is exactly only one call to this function or any other
    /// accessor of the RWLock until the returned borrow goes out of scope.
    pub unsafe fn as_ref_unchecked(&self) -> &T {
        &*self.data.get()
    }
}

impl<T> fmt::Debug for RWLock<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataLock")
            .field("Value", unsafe { &*self.data.get() })
            .field("ReadLocks", &self.read_locks)
            .finish()
    }
}

// when the WriteLockGuard is dropped release the owning lock
impl<T> Drop for WriteLockGuard<'_, T> {
    fn drop(&mut self) {
        self._data.write_lock.store(false, Ordering::Release);
        //println!("write lock released {:?}", core::any::type_name::<T>());

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

// when the ReadLockGuard is dropped release the owning lock
impl<T> Drop for ReadLockGuard<'_, T> {
    fn drop(&mut self) {
        self._data.read_locks.fetch_sub(1, Ordering::Release);
        //println!("read lock released {:?}", core::any::type_name::<T>());

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
// be ensured. Therefore also only one ``WriteLockGuard`` could ever exist for one specific ``RWLock``, which makes
// it safe to return immutable and mutable references.
impl<T> Deref for WriteLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self._data.data.get() }
    }
}

impl<T> DerefMut for WriteLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self._data.data.get() }
    }
}

// the ``ReadLockGuard`` can only be immutable dereferenced
impl<T> Deref for ReadLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self._data.data.get() }
    }
}

/// The RWLock is always `Sync`, to make it `Send` as well it need to be wrapped into an `Arc`.
unsafe impl<T> Sync for RWLock<T> {}
