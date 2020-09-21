/***********************************************************************************************************************
 * Copyright (c) 2020 by the authors
 *
 * Author: André Borrmann <pspwizard@gmx.de>
 * License: Apache License 2.0 / MIT
 **********************************************************************************************************************/

 //! # Sync Locking
 //!
 //!

mod spinlock;
#[doc(inline)]
pub use spinlock::*;

// re-export the semaphore
mod semaphore;
#[doc(inline)]
pub use semaphore::*;

// re-export the data-lock
mod mutex;
#[doc(inline)]
pub use mutex::*;

// re-export the data read/write lock
mod rwlock;
pub use rwlock::*;