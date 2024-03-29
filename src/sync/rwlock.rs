/***********************************************************************************************************************
 * Copyright (c) 2020 by the authors
 *
 * Author: André Borrmann <pspwizard@gmx.de>
 * License: Apache License 2.0 / MIT
 **********************************************************************************************************************/

//! # RWLock
//!

use core::arch::asm;
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
pub struct WriteLockGuard<'a, T: ?Sized + 'a> {
  _data: &'a RWLock<T>,
}

/// Result of aquiring read access to the data using ``read`` on the data lock. If the
/// result goes out of scope the read lock is released.
pub struct ReadLockGuard<'a, T: ?Sized + 'a> {
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
}

impl<T: ?Sized> RWLock<T> {
  /// Try to provide a Writelock for mutual exclusive access. Returns ``None`` if the lock fails
  /// or ``Some(WriteLockGuard)``. The actual data, the [WriteLockGuard] wraps could be conviniently accessed by
  /// dereferencing it.
  pub fn try_write(&self) -> Option<WriteLockGuard<T>> {
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
        asm!("dmb sy");
      }

      Some(WriteLockGuard { _data: self })
    } else {
      // we couldn't set the lock
      None
    }
  }

  /// Provide a WriteLock for mutual exclusive access. This blocks until the data could be
  /// successfully locked. This also implies that there is no concurrent [ReadLockGuard] existing.
  /// The locked data will be returned as [WriteLockGuard]. Simply derefrencing
  /// this allows access to the contained data value.
  ///
  pub fn write(&self) -> WriteLockGuard<T> {
    loop {
      if let Some(write_guard) = self.try_write() {
        //println!("write lock aquired {:?}", core::any::type_name::<T>());
        return write_guard;
      }
      // to save energy and cpu consumption we can wait for an event beeing raised that indicates that the
      // semaphore value has likely beeing changed
      #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
      unsafe {
        asm!("wfe");
      }
    }
  }

  /// Provide a ReadLock to the wrapped data. This call blocks until the recource is available.
  /// There can be as many concurrent [ReadLockGuard]s being handed out if there is no [WriteLockGuard] to the
  /// same resource already existing.
  pub fn try_read(&self) -> Option<ReadLockGuard<T>> {
    // read locks can only handed out if no write lock is existing already
    if self.write_lock.load(Ordering::Relaxed) {
      None
    } else {
      self.read_locks.fetch_add(1, Ordering::Acquire);
      //println!("read lock aquired {:?}", core::any::type_name::<T>());
      Some(ReadLockGuard { _data: self })
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
        asm!("wfe");
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

  /// Consume the Mutex and return the inner value
  pub fn into_inner(self) -> T
  where
    T: Sized,
  {
    self.data.into_inner()
  }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for RWLock<T>
where
  T: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut dbg = f.debug_struct("DataLock");
    match self.try_read() {
      Some(guard) => {
        dbg.field("Value", &&*guard);
      }
      _ => {
        dbg.field("Value", &"unable to r-lock");
      }
    }
    dbg.field("ReadLocks", &self.read_locks);
    dbg.finish_non_exhaustive()
  }
}

// when the WriteLockGuard is dropped release the owning lock
impl<T: ?Sized> Drop for WriteLockGuard<'_, T> {
  fn drop(&mut self) {
    self._data.write_lock.store(false, Ordering::Release);
    //println!("write lock released {:?}", core::any::type_name::<T>());

    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    unsafe {
      // dmb required before allow access to the protected resource, see:
      // http://infocenter.arm.com/help/topic/com.arm.doc.dht0008a/DHT0008A_arm_synchronization_primitives.pdf
      asm!("dmb sy");
      // also raise a signal to indicate the semaphore has been changed (this trigger all WFE's to continue
      // processing) but do data syncronisation barrier upfront to ensure any data updates has been finished
      asm!(
        "dsb sy
         sev"
      );
    }
  }
}

// when the ReadLockGuard is dropped release the owning lock
impl<T: ?Sized> Drop for ReadLockGuard<'_, T> {
  fn drop(&mut self) {
    self._data.read_locks.fetch_sub(1, Ordering::Release);
    //println!("read lock released {:?}", core::any::type_name::<T>());

    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    unsafe {
      // dmb required after atomic operations, see:
      // http://infocenter.arm.com/help/topic/com.arm.doc.dht0008a/DHT0008A_arm_synchronization_primitives.pdf
      asm!("dmb sy");
    }
  }
}

// dereferencing the value contained in the DataWriteLock
// this is ok as the DataWriteLock does only exist if the exclusive access to the data could
// be ensured. Therefore also only one ``WriteLockGuard`` could ever exist for one specific ``RWLock``, which makes
// it safe to return immutable and mutable references.
impl<T: ?Sized> Deref for WriteLockGuard<'_, T> {
  type Target = T;

  fn deref(&self) -> &T {
    unsafe { &*self._data.data.get() }
  }
}

impl<T: ?Sized> DerefMut for WriteLockGuard<'_, T> {
  fn deref_mut(&mut self) -> &mut T {
    unsafe { &mut *self._data.data.get() }
  }
}

// the ``ReadLockGuard`` can only be immutable dereferenced
impl<T: ?Sized> Deref for ReadLockGuard<'_, T> {
  type Target = T;

  fn deref(&self) -> &T {
    unsafe { &*self._data.data.get() }
  }
}

/// implement debug trait to forward to the type wrapped within the guard
impl<T: ?Sized + fmt::Debug> fmt::Debug for WriteLockGuard<'_, T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Debug::fmt(&**self, f)
  }
}

/// implement debug trait to forward to the type wrapped within the guard
impl<T: ?Sized + fmt::Debug> fmt::Debug for ReadLockGuard<'_, T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Debug::fmt(&**self, f)
  }
}

/// The RWLock is always `Sync`, to make it `Send` as well it need to be wrapped into an `Arc`.
unsafe impl<T: ?Sized + Send> Sync for RWLock<T> {}

#[cfg(testing)]
mod tests {
  extern crate alloc;
  use super::*;
  use alloc::sync::Arc;

  #[test]
  fn only_one_write_lock() {
    let rwlock = Arc::new(RWLock::new(0u32));
    let rwlock_clone = Arc::clone(&rwlock);
    // try_lock and lock will provide a WriteLock
    let mut data = rwlock.write();
    *data = 20;
    // if a write lock exists no read lock's could be aquired
    assert!(rwlock_clone.try_write().is_none());
  }

  #[test]
  fn only_one_write_no_readlock() {
    let rwlock = Arc::new(RWLock::new(0u32));
    let rwlock_clone = Arc::clone(&rwlock);
    // try_lock and lock will provide a WriteLock
    let mut data = rwlock.write();
    *data = 20;
    // if a write lock exists no read lock's could be aquired
    assert!(rwlock_clone.try_read().is_none());
  }

  #[test]
  fn only_multiple_readlocks() {
    let rwlock = Arc::new(RWLock::new(0u32));
    let rwlock_clone = Arc::clone(&rwlock);
    // try_lock and lock will provide a WriteLock
    let data = rwlock.read();
    // if a write lock exists no read lock's could be aquired
    assert!(rwlock_clone.try_read().is_some());
    println!("{}", *data);
  }

  #[test]
  fn only_read_no_write_lock() {
    let rwlock = Arc::new(RWLock::new(0u32));
    let rwlock_clone = Arc::clone(&rwlock);
    // try_lock and lock will provide a WriteLock
    let data = rwlock.read();
    // if a write lock exists no read lock's could be aquired
    assert!(rwlock_clone.try_write().is_none());
    println!("{}", *data);
  }
}
