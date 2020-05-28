use crate::i2c_proxy::I2cHandlerProxy;
use cortex_m_semihosting::hprintln;
use crate::scoped_task_spawn_provider::ExposedI2cProxy;
use crate::i2c_proxy::I2cHandlerCallable;

pub struct SomeDriver<I2C>
    where
        I2C: embedded_hal::blocking::i2c::Read,
{
    i2c: I2C,
}

impl<I2C, TErr> SomeDriver<I2C>
    where I2C: embedded_hal::blocking::i2c::Read<Error=TErr>, TErr: core::fmt::Debug {
    pub fn new(i2c: I2C) -> Self {
        SomeDriver { i2c }
    }

    pub fn do_stuff(&mut self) {
        hprintln!("SomeDriver: Read from I2c via e_h `Read` trait!").unwrap();
        let mut buffer_to_filled = [0_u8; 100];
        hprintln!(
            "SomeDriver: buffer content before call: {:#X}, {:#X}, ...",
            buffer_to_filled[0], buffer_to_filled[1]).unwrap();
        self.i2c.read(0xff, &mut buffer_to_filled).unwrap();
        hprintln!(
            "SomeDriver: buffer content after call: {:#X}, {:#X}, ...",
            buffer_to_filled[0], buffer_to_filled[1]).unwrap();
    }
}

impl<T: I2cHandlerCallable> ExposedI2cProxy<T> for SomeDriver<I2cHandlerProxy<T>> {
    fn expose(&mut self) -> &mut I2cHandlerProxy<T> {
        &mut self.i2c
    }
}
