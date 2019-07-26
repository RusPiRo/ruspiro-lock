/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-lock/0.0.1")]
#![no_std]
#![feature(asm)]

//! # Atomic locks for Raspberry Pi baremetal
//! 

pub mod spinlock;
pub use self::spinlock::*;

pub mod semaphore;
pub use self::semaphore::*;