pub trait I2cHandlerCallable {
    fn call_i2c(&self, command: I2cCommand) -> Result<(), I2cCommand>;
}

struct SpawnUnsafe<T: I2cHandlerCallable>(Option<*const T>);

unsafe impl<T: I2cHandlerCallable> Send for SpawnUnsafe<T> {}

#[derive(Debug)]
pub enum I2cCommand {
    Write(u8, &'static [u8]),
    Read(u8, &'static mut [u8]),
    WriteRead(u8, &'static [u8], &'static mut [u8]),
}

pub struct I2cHandlerProxy<T: I2cHandlerCallable> {
    spawn: SpawnUnsafe<T>,
}

impl<T: I2cHandlerCallable> I2cHandlerProxy<T> {
    pub fn new() -> Self {
        I2cHandlerProxy {
            spawn: SpawnUnsafe(None),
        }
    }

    pub fn set_spawn<U: I2cHandlerCallable>(&mut self, spawn: Option<*const U>) {
        self.spawn = SpawnUnsafe(spawn.map(|val| unsafe { core::mem::transmute(val) }));
    }
}

impl<T: I2cHandlerCallable> embedded_hal::blocking::i2c::Write for I2cHandlerProxy<T> {
    type Error = I2cCommand;

    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        match self.spawn.0 {
            Some(spawn) => unsafe {
                spawn
                    .as_ref()
                    .unwrap()
                    .call_i2c(I2cCommand::Write(addr, core::mem::transmute(bytes)))
            },
            None => panic!("Spawn object not available")
        }
    }
}

impl<T: I2cHandlerCallable> embedded_hal::blocking::i2c::Read for I2cHandlerProxy<T> {
    type Error = I2cCommand;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        match self.spawn.0 {
            Some(spawn) => unsafe {
                spawn
                    .as_ref()
                    .unwrap()
                    .call_i2c(I2cCommand::Read(
                        address,
                        core::mem::transmute(buffer)))
            },
            None => panic!("Spawn object not available")
        }
    }
}

impl<T: I2cHandlerCallable> embedded_hal::blocking::i2c::WriteRead for I2cHandlerProxy<T> {
    type Error = I2cCommand;

    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        match self.spawn.0 {
            Some(spawn) => unsafe {
                spawn
                    .as_ref()
                    .unwrap()
                    .call_i2c(I2cCommand::WriteRead(
                        address,
                        core::mem::transmute(bytes),
                        core::mem::transmute(buffer)))
            },
            None => panic!("Spawn object not available")
        }
    }
}
