#![no_std]

extern crate pin_api;
extern crate cortex_m;
extern crate futures_core as futures;
extern crate futures_stable as stable;

use core::u32;
use core::{ptr, convert::From};

use futures::task::{Context, LocalMap, Waker, UnsafeWake};
use futures::{Async, Future, IntoFuture};

struct WFEWaker;

unsafe impl UnsafeWake for WFEWaker {
    unsafe fn clone_raw(&self) -> Waker {
        Waker::from(WFEWaker)
    }

    unsafe fn drop_raw(&self) {
        // No-op, we're a ZST and just use NULL as our pointer
    }

    unsafe fn wake(&self) {
        // No-op, we use WFE instructions instead
    }
}

impl From<WFEWaker> for Waker {
    fn from(_: WFEWaker) -> Waker {
        unsafe {
            Waker::new(ptr::null_mut() as *mut WFEWaker as *mut UnsafeWake)
        }
    }
}

pub struct Executor(cortex_m::Peripherals);

impl Executor {
    pub fn new(peripherals: cortex_m::Peripherals) -> Executor {
        // enable WFE
        unsafe {
            peripherals.SCB.scr.modify(|x| (x | 0b00010000));
        }

        Executor(peripherals)
    }

    pub fn run<F: IntoFuture>(self, future: F) -> Result<F::Item, F::Error> {
        let mut map = LocalMap::new();
        let waker = Waker::from(WFEWaker);
        let mut context = Context::new(&mut map, &waker);
        let mut future = future.into_future();
        loop {
            match future.poll(&mut context) {
                Ok(Async::Pending) => {}
                Ok(Async::Ready(val)) => {
                    return Ok(val);
                }
                Err(err) => {
                    return Err(err);
                }
            }
            // Clear all pending interrupts
            unsafe {
                for i in 0..16 {
                    self.0.NVIC.icpr[i].write(u32::MAX);
                }
            }
            cortex_m::asm::wfe();
        }
    }
}
