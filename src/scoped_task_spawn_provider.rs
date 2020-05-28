use crate::i2c_proxy::{I2cHandlerProxy, I2cHandlerCallable};
use cortex_m_semihosting::hprintln;
use core::cell::RefCell;
use core::marker::PhantomData;

pub trait ExposedI2cProxy<T: I2cHandlerCallable> {
    fn expose(&mut self) -> &mut I2cHandlerProxy<T>;
}

pub struct ScopedTaskSpawnProvider<'a, T, U, W>
    where
        T: ExposedI2cProxy<U>,
        U: I2cHandlerCallable,
        W: I2cHandlerCallable {
    driver_with_proxy: &'a RefCell<T>,
    _u: PhantomData<U>,
    _w: PhantomData<W>,
}

impl<'a, T, U, W> ScopedTaskSpawnProvider<'a, T, U, W>
    where
        T: ExposedI2cProxy<U>,
        U: I2cHandlerCallable,
        W: I2cHandlerCallable {
    pub fn new(spawn: &'a W, driver_with_proxy: &'a RefCell<T>) -> Self {
        hprintln!("STSP: Created a scope with `Spawn` struct reference!").unwrap();
        driver_with_proxy.borrow_mut().expose().set_spawn(Some(spawn));
        ScopedTaskSpawnProvider { driver_with_proxy, _u: PhantomData, _w: PhantomData }
    }
}

impl<'a, T, U, W> Drop for ScopedTaskSpawnProvider<'a, T, U, W>
    where
        T: ExposedI2cProxy<U>,
        U: I2cHandlerCallable,
        W: I2cHandlerCallable {
    fn drop(&mut self) {
        hprintln!("STSP: Drop called, `Spawn` reference set to None").unwrap();
        self.driver_with_proxy.borrow_mut().expose().set_spawn(Option::<*const W>::None);
    }
}
