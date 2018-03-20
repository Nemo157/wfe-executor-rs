use core::marker::PhantomData;

use pin_api::mem::Pin;

pub struct PinTemporary<'a, T: 'a> {
    data: T,
    _marker: PhantomData<&'a &'a mut ()>,
}

pub fn pinned<'a, T: 'a>(data: T) -> PinTemporary<'a, T> {
    PinTemporary { data, _marker: PhantomData }
}

impl<'a, T> PinTemporary<'a, T> {
    pub fn as_pin(&'a mut self) -> Pin<'a, T> {
        unsafe { Pin::new_unchecked(&mut self.data) }
    }
}

