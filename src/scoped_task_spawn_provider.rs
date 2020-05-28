use crate::i2c_proxy::{I2cHandlerProxy, I2cHandlerCallable};
use cortex_m_semihosting::hprintln;
use core::cell::RefCell;
use core::marker::PhantomData;

// Trait that must be implemented by driver creator.
// This is bad, as any person trying to use this solution must
// fork driver and add proper trait implementation.
// I didn't manage to figure out how I can expose underlying
// i2c proxy that is hidden in driver without changing driver
// implementation.
pub trait ExposedI2cProxy<T: I2cHandlerCallable> {
    fn expose(&mut self) -> &mut I2cHandlerProxy<T>;
}

// This double `I2cHandlerCallable` generic parameter is needed,
// because I cannot reuse `U` type for `spawn` parameter.
// That is because compiler associates `U` with `T` and calling `STSP::new()`
// with `driver` (type of which is declared with `static` lifetime in `Resources`)
// will require passed `spawn` object to have `static` lifetime as well - which is
// not the case.
// I perform nasty lifetime erasure inside the proxy.
// This double generic param is unsound as hell, as it allows - as mentioned
// in main.rs - to pass any object implementing trait `I2cHandlerCallable` to STSP
// and it will be casted bitwise by `core::mem::transmute` in I2cHandlerProxy::set_spawn.
// To circumvent this I can use concrete `Spawn` type here but then
// this module will become dependent upon RTIC's `Spawn` types which is undesired :(
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
        // Setting `Spawn` object on creation.
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
        // Dropping `Spawn` object on destruction.
        self.driver_with_proxy.borrow_mut().expose().set_spawn(Option::<*const W>::None);
    }
}
