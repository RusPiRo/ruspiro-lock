/***********************************************************************************************************************
 * Copyright (c) 2020 by the authors
 *
 * Author: André Borrmann <pspwizard@gmx.de>
 * License: Apache License 2.0 / MIT
 **********************************************************************************************************************/

//! # Async Semaphore
//!

extern crate alloc;

use crate::sync::{Mutex, Semaphore};
use alloc::{collections::BTreeMap, sync::Arc};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

pub struct AsyncSemaphore {
    inner: Arc<Mutex<AsyncSemaphoreInner>>,
    sema: Arc<Semaphore>,
}

impl AsyncSemaphore {
    pub fn new(initial: u32) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AsyncSemaphoreInner::new())),
            sema: Arc::new(Semaphore::new(initial)),
        }
    }

    pub async fn down(&self) {
        // if we cann't immediately pull the semaphore down we need to use a future to poll the
        // result
        if self.sema.try_down().is_err() {
            let mut inner = self.inner.lock();
            let current_id = inner.next_waiter;
            inner.next_waiter += 1;
            drop(inner);

            AsyncSemaphoreFuture::new(Arc::clone(&self.inner), Arc::clone(&self.sema), current_id)
                .await
        }
    }

    /// when increasing the [AsyncSemaphore] we will increase the embedded [Semaphore] and notify the next waiter in the
    /// list that previously did not got the chance to decrease the [Semaphore]
    pub fn up(&self) {
        self.sema.up();

        let mut inner = self.inner.lock();
        if let Some(&waiter_id) = inner.waiter.keys().next() {
            let waiter = inner.waiter.remove(&waiter_id).unwrap();
            waiter.wake();
        }
    }
}

/// The `Future` that represents an `await`able semaphore down request to an [AsyncSemaphore] and can only be created 
/// from functions of the [AsyncSemaphore]
struct AsyncSemaphoreFuture {
    inner: Arc<Mutex<AsyncSemaphoreInner>>,
    sema: Arc<Semaphore>,
    id: usize,
}

impl AsyncSemaphoreFuture {
    fn new(inner: Arc<Mutex<AsyncSemaphoreInner>>, sema: Arc<Semaphore>, id: usize) -> Self {
        Self { inner, sema, id }
    }
}

impl Future for AsyncSemaphoreFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if this.sema.try_down().is_ok() {
            Poll::Ready(())
        } else {
            let mut inner = this.inner.lock();
            inner.waiter.insert(this.id, cx.waker().clone());
            drop(inner);

            Poll::Pending
        }
    }
}

struct AsyncSemaphoreInner {
    /// If the lock could not be aquired we store the requestor id here to allow the next one
    /// already waiting for the lock to retrieve it
    waiter: BTreeMap<usize, Waker>,
    /// The id of the next waiter that can be woken once the lock is released and someone else is already waiting for
    /// the lock to be aquired
    next_waiter: usize,
}

impl AsyncSemaphoreInner {
    fn new() -> Self {
        Self {
            waiter: BTreeMap::new(),
            next_waiter: 0,
        }
    }
}
