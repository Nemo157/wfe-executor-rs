#![no_std]
#![feature(conservative_impl_trait)]
#![feature(never_type)]
#![feature(duration_extras)]
#![feature(generators)]
#![feature(proc_macro)]

extern crate anchor_experiment;
extern crate cortex_m;
extern crate futures_core as futures;
extern crate futures_stable as stable;

use core::{ptr, convert::From};

use anchor_experiment::{Pin, pinned};
use futures::task::{Context, LocalMap, Waker, UnsafeWake};
use futures::{Async, Future, IntoFuture};
use stable::StableFuture;

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

    pub fn run<F>(self, future: F) where F: IntoFuture<Item = !, Error = !> {
        let mut map = LocalMap::new();
        let waker = Waker::from(WFEWaker);
        let mut context = Context::new(&mut map, &waker);
        let mut future = future.into_future();
        loop {
            let Ok(Async::Pending) = future.poll(&mut context);
            // self.0.NVIC.clear_pending(...);
            cortex_m::asm::wfe();
        }
    }

    pub fn run_stable<F>(self, future: F) where F: StableFuture<Item = !, Error = !> {
        let mut map = LocalMap::new();
        let waker = Waker::from(WFEWaker);
        let mut context = Context::new(&mut map, &waker);
        let mut future = pinned(future);
        let mut future = future.as_pin();
        loop {
            let Ok(Async::Pending) = Pin::borrow(&mut future).poll(&mut context);
            // self.0.NVIC.clear_pending(...);
            cortex_m::asm::wfe();
        }
    }
}
