file ./target/thumbv7em-none-eabihf/debug/shared-i2c-rtic-exercise
#target extended-remote :4242  
# st-util

target extended-remote :3333
# openocd
monitor arm semihosting enable
# For openocd only

# print demangled symbols
set print asm-demangle on

# Show numeral variable in hexadecimal format
set output-radix 16

# set backtrace limit to not have infinite backtrace loops
set backtrace limit 32

# detect unhandled exceptions, hard faults and panics

#break DefaultHandler
#break HardFault
#break rust_begin_unwind

load

# start the process but immediately halt the processor
stepi
