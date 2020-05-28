#![no_std]
#![no_main]

mod some_driver;
mod i2c_proxy;
mod scoped_task_spawn_provider;

use panic_semihosting as _;
use cortex_m_semihosting::hprintln;
use rtfm::app;

use stm32f4xx_hal::rcc::RccExt;
use stm32f4xx_hal::time::U32Ext;
use some_driver::SomeDriver;
use i2c_proxy::I2cHandlerProxy;
use i2c_proxy::I2cCommand;

use crate::scoped_task_spawn_provider::ScopedTaskSpawnProvider;
use core::cell::RefCell;
use crate::i2c_proxy::I2cHandlerCallable;

#[app(device = stm32f4::stm32f407, peripherals = true, monotonic = rtfm::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        driver:
        RefCell<
            SomeDriver<
                I2cHandlerProxy<
                    crate::some_driver_handler::Spawn<'static>
                >
            >
        >,
    }
    #[init(spawn = [some_driver_handler])]
    fn init(mut cx: init::Context) -> init::LateResources {
        let _clocks = cx.device.RCC.constrain().cfgr.sysclk(168.mhz()).freeze();
        cx.core.DCB.enable_trace();
        cx.core.DWT.enable_cycle_counter();

        let driver =
            RefCell::new(
                SomeDriver::new(
                    I2cHandlerProxy::new()));

        cx.spawn.some_driver_handler().unwrap();
        init::LateResources { driver }
    }

    #[task(resources = [driver], spawn = [i2c_handler])]
    fn some_driver_handler(cx: some_driver_handler::Context) {
        let driver = cx.resources.driver;
        hprintln!("some_driver_handler: called!").unwrap();
        let p = ScopedTaskSpawnProvider::new(&cx.spawn, driver);

        driver.borrow_mut().do_stuff();
        let _p = p;
        hprintln!("some_driver_handler: finished!").unwrap();
    }

    #[task(resources = [driver])]
    fn handler(cx: handler::Context) {
        // This should panic, as internal proxy hold Option::None with regards to `Spawn` ptr.
        hprintln!("out-of-scope call to driver will panic!").unwrap();
        cx.resources.driver.borrow_mut().do_stuff();
    }

    #[task(spawn = [handler], priority = 2)]
    fn i2c_handler(cx: i2c_handler::Context, command: I2cCommand) {
        match command {
            I2cCommand::Write(address, bytes) =>
                hprintln!("i2c_handler: Write | address: {:?}, bytes: {:?}!", address, bytes),
            I2cCommand::Read(address, buffer) =>
                hprintln!("i2c_handler: Read | address: {:?}, buffer: {:?}!", address, buffer),
            I2cCommand::WriteRead(address, bytes, buffer) =>
                hprintln!("i2c_handler: WriteRead | address: {:?}, bytes: {:?}, buffer {:?}!", address, bytes, buffer),
        }.unwrap();
        cx.spawn.handler().unwrap();
        hprintln!("i2c_handler: finished!").unwrap();
    }

    extern "C" {
        fn ADC();
        fn USART1();
    }
};

impl<'a> I2cHandlerCallable for crate::some_driver_handler::Spawn<'a> {
    fn call_i2c(&self, command: I2cCommand) -> Result<(), I2cCommand> {
        self.i2c_handler(command)
    }
}
