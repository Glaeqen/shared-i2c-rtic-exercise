#![no_std]
#![no_main]

mod some_driver;
mod i2c_proxy;
mod scoped_task_spawn_provider;

use panic_semihosting as _;
use cortex_m_semihosting::hprintln;
use rtfm::app;

use some_driver::SomeDriver;
use i2c_proxy::I2cHandlerProxy;
use i2c_proxy::I2cCommand;

use crate::scoped_task_spawn_provider::ScopedTaskSpawnProvider;
use core::cell::RefCell;
use crate::i2c_proxy::I2cHandlerCallable;

#[app(device = lm3s6965, peripherals = true)]
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
        // Explanation regarding `driver` type:
        // - `RefCell` is used because I need the ability to hold two mutable references
        // at once - one for STSP (ScopedTaskSpawnProvider, explained later) and one obviously
        // for a driver user.
        // - `SomeDriver` is a mockup of some generic driver using I2C consuming underlying i2c
        // device.
        // - `I2cHandlerProxy` is a type pretending to be an i2c device (similarly to what
        // `shared-bus` crate is trying to achieve) which spawns a i2c-handling task when driver
        // calls it.
        // - `Spawn` is used by `I2cHandlerProxy` to spawn tasks. Here concrete types are required
        // so I had to specify `Spawn` type with `static` lifetime.
    }
    #[init(spawn = [some_driver_handler])]
    fn init(cx: init::Context) -> init::LateResources {
        let driver =
            RefCell::new(
                SomeDriver::new(
                    I2cHandlerProxy::new()));

        cx.spawn.some_driver_handler().unwrap();
        init::LateResources { driver }
    }

    #[task(resources = [driver], spawn = [i2c_handler, invalid_handler])]
    fn some_driver_handler(cx: some_driver_handler::Context) {
        hprintln!("some_driver_handler: called!").unwrap();
        // Here, I'm instantiating STSP type which is valid till the end of this function.
        // On creation, it populates `Spawn` struct inside of `driver`'s `I2cHandlerProxy`
        let _scope = ScopedTaskSpawnProvider::new(&cx.spawn, cx.resources.driver);

        // Following line will cause `i2c_handler` to preempt this task
        cx.resources.driver.borrow_mut().do_stuff();
        // Spawning another task trying to access driver
        // Note that it will be called after this task finishes as they have equal priorities :)
        cx.spawn.invalid_handler().unwrap();
        hprintln!("some_driver_handler: finished!").unwrap();
        // On drop, it sets `Spawn` reference inside of `driver`'s `I2cHandlerProxy` to `None`.
        // Now, if someone tried to use `driver`, it would panic.
    }

    #[task(resources = [driver], spawn = [i2c_handler])]
    fn invalid_handler(cx: invalid_handler::Context) {
        // This should panic, as internal proxy hold Option::None when it comes to `Spawn` ptr.
        hprintln!("invalid_handler: out-of-scope call to driver is going to panic!").unwrap();
        // let _scope = ScopedTaskSpawnProvider::new(&cx.spawn, cx.resources.driver);
        // UNSOUNDNESS issue:
        // Previous commented line is unsound, as I can provide any object implementing
        // `I2cHandlerCallable`.
        // Actually, (i'm not sure) passing RTIC-generated `Spawn` objects somewhat works out,
        // as these presumably have the same structure bitwise. Still UB-ish.
        cx.resources.driver.borrow_mut().do_stuff();
    }

    #[task(priority = 2)]  // Note: high priority
    fn i2c_handler(_: i2c_handler::Context, command: I2cCommand) {
        // UNSOUNDNESS issue:
        // For this solution to be valid, `i2c_handler` MUST have higher priority than every
        // calling task AND `command` must not be passed to any other task with lower priority
        // than `i2c_handler` task.
        match command {
            I2cCommand::Write(_address, _bytes) =>
                unimplemented!("Write unimplemented!"),
            I2cCommand::Read(address, buffer) => {
                for val in buffer {
                    *val = 0xAE;
                }
                hprintln!("i2c_handler: Read | address: {:?}!", address)
            }
            I2cCommand::WriteRead(_address, _bytes, _buffer) =>
                unimplemented!("WriteRead unimplemented!"),
        }.unwrap();
    }

    extern "C" {
        fn GPIOA();
        fn GPIOB();
    }
};

// For STSP to work with any `Spawn` type, I introduced `I2cHandlerCallable` trait that has to be
// implemented for any `Spawn` object that is supposed to provide i2c task spawning capabilities
impl<'a> I2cHandlerCallable for crate::some_driver_handler::Spawn<'a> {
    fn call_i2c(&self, command: I2cCommand) -> Result<(), I2cCommand> {
        self.i2c_handler(command)
    }
}
