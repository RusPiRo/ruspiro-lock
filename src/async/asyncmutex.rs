/***********************************************************************************************************************
 * Copyright (c) 2020 by the authors
 *
 * Author: André Borrmann <pspwizard@gmx.de>
 * License: Apache License 2.0 / MIT
 **********************************************************************************************************************/

//! # Async Mutex
//!

extern crate alloc;
use crate::sync::{Mutex, MutexGuard};
use alloc::{collections::BTreeMap, sync::Arc};
use core::{
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll, Waker},
};

/// An async mutex lock that can be used in async functions to prevent blocking current execution while waiting for the
/// lock to become available. So for this to work the `lock()` method does not return a MutexGuard immediately but a
/// [Future] that will resove into a [AsyncMutexGuard] when `await`ed.
pub struct AsyncMutex<T> {
    /// The inner wrapper to the actual [Mutex] requires to be secured with a [Mutex] on it's own
    /// as we require mutual exclusive access to it. This actually should not harm any concurrent blocking
    /// as this is a short living lock that will be only aquired to request the actual lock status. So it is
    /// more then unlikely that this will happen in parallel at the same time
    inner: Arc<Mutex<AsyncMutexInner>>,
    /// The actual [Mutex] securing the contained data for mutual exclusive access
    data: Arc<Mutex<T>>,
}

impl<T> AsyncMutex<T> {
    /// Create the [AsyncMutex]
    pub fn new(value: T) -> Self {

        Self {
            inner: Arc::new(Mutex::new(AsyncMutexInner::new())),
            data: Arc::new(Mutex::new(value)),
        }
    }

    /// Locking the data secured by the [AsyncMutex] will yield a `Future` that must be awaited to actually acquire
    /// the lock.
    pub async fn lock(&self) -> AsyncMutexGuard<'_, T> {
        // check if we could immediately get the lock
        if let Some(guard) = self.data.try_lock() {
            // lock immediatly acquired, provide the lock guard as result
            AsyncMutexGuard {
                guard,
                inner: Arc::clone(&self.inner),
            }
        } else {
            // to be able to request the lock we require to upate the inner metadata. For this to work we require a
            // short living exclusive lock to this data.
            let mut inner = self.inner.lock();
            let current_id = inner.next_waiter;
            inner.next_waiter += 1;
            drop(inner);

            // once we have updated the metadata we can release the lock to it and create the `Future` that will yield
            // the lock to the data once available
            AsyncMutexFuture::new(
                Arc::clone(&self.inner),
                Arc::clone(&self.data),
                current_id,
            )
            .await
        }
    }
}

pub struct AsyncMutexGuard<'a, T> {
    guard: MutexGuard<'a, T>,
    inner: Arc<Mutex<AsyncMutexInner>>,
}

impl<'a, T> Deref for AsyncMutexGuard<'a, T> {
    type Target = MutexGuard<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a, T> DerefMut for AsyncMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

/// If an [AsyncMutexGuard] get's dropped we need to wake the `Future`s that might hav registered themself and
/// are waiting to aquire the lock.
impl<T> Drop for AsyncMutexGuard<'_, T> {
    fn drop(&mut self) {
        // if the mutex guard is about to be locked we need to check if there has been a waker send
        // already to get woken
        let mut inner = self.inner.lock();
        if let Some(&next_waiter) = inner.waiter.keys().next() {
            // remove the waker from the waiter list as it will re-register itself when the corresponding
            // Future is polled and can't acquire the lock
            let waiter = inner
                .waiter
                .remove(&next_waiter)
                .expect("found key but can't remove it ???");
            waiter.wake();
        }
    }
}

/// The `Future` that represents an `await`able [AsynMutex] and can only be created from the functions of [AsyncMutex].
struct AsyncMutexFuture<'a, T: ?Sized> {
    inner: Arc<Mutex<AsyncMutexInner>>,
    data: Arc<Mutex<T>>,
    id: usize,
    _p: core::marker::PhantomData<&'a T>,
}

impl<T> AsyncMutexFuture<'_, T> {
    fn new(
        inner: Arc<Mutex<AsyncMutexInner>>,
        data: Arc<Mutex<T>>,
        id: usize,
    ) -> Self {
        Self {
            inner,
            data,
            id,
            _p: core::marker::PhantomData,
        }
    }
}

impl<'a, T> Future for AsyncMutexFuture<'a, T> {
    type Output = AsyncMutexGuard<'a, T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // we need to elide the lifetime given by self.get_mut() using unsafe code here
        // SAFETY: it's actually safe as we either return Poll::Pending without any lifetime or we
        // handout the `AsyncMutexGuard` with lifetime 'a which bound to the AsyncMutex that created this Future and
        // will always outlive this future and is therefore ok - I guess...
        let this = unsafe { &*(self.get_mut() as *const Self) };
        if let Some(guard) = this.data.try_lock() {
            // data lock could be acquired
            // provide the AsyncMutexGuard
            Poll::Ready(AsyncMutexGuard {
                guard,
                inner: Arc::clone(&this.inner),
            })
        } else {
            // data lock could not be acquired this time, so someone else is holding the lock. We need to register
            // ourself to get woken as soon as the lock gets available
            let mut inner = this.inner.lock();
            inner.waiter.insert(this.id, cx.waker().clone());
            drop(inner);

            Poll::Pending
        }
    }
}
struct AsyncMutexInner {
    /// If the lock could not be aquired we store the requestor id here to allow the next one
    /// already waiting for the lock to retrieve it
    waiter: BTreeMap<usize, Waker>,
    /// The id of the next waiter that can be woken once the lock is released and someone else is already waiting for
    /// the lock to be aquired
    next_waiter: usize,
}

impl AsyncMutexInner {
    fn new() -> Self {
        Self {
            waiter: BTreeMap::new(),
            next_waiter: 0,
        }
    }
}


#[cfg(test)]
mod tests {
    use async_std::prelude::*;
    use async_std::task;
    use core::time::Duration;
    use super::*;

    #[async_std::test]
    async fn wait_on_mutex() {
        let mutex = Arc::new(AsyncMutex::new(10_u32));
        let mutex_clone = Arc::clone(&mutex);
        
        let task1 = task::spawn(async move {
            let mut guard = mutex_clone.lock().await;
            **guard = 20;
            // with the AsyncMutexLock in place wait a second to keep the guard
            // alive and let the second task relly wait for this one
            task::yield_now().await;
            task::sleep(Duration::from_secs(1)).await;
        });

        let task2 = task::spawn( async move {
            // if this async is started first wait a bit to really run the
            // other one first to aquire the AsyncMutexLock
            task::yield_now().await;
            task::sleep(Duration::from_millis(100)).await;
            let guard = mutex.lock().await;
            let value = **guard;
            assert_eq!(20, value);
        });
        
        // run both tasks concurrently
        task1.join(task2).await;
    }
}