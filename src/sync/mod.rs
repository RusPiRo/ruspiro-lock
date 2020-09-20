/***********************************************************************************************************************
 * Copyright (c) 2020 by the authors
 *
 * Author: André Borrmann <pspwizard@gmx.de>
 * License: Apache License 2.0 / MIT
 **********************************************************************************************************************/

mod spinlock;
pub use spinlock::*;

// re-export the semaphore
mod semaphore;
pub use semaphore::*;

// re-export the data-lock
mod mutex;
pub use mutex::*;

// re-export the data read/write lock
mod rwlock;
pub use rwlock::*;