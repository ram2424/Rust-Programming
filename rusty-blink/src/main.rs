#![deny(unsafe_code)]
#![no_main]
#![no_std]


// Halt on panic
use panic_halt as _;


use cortex_m_rt::entry;
use cortex_m::delay::Delay;
use stm32g4xx_hal::{
    pac,
    prelude::*,
    rcc::Config,
    stm32,
};

#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32::Peripherals::take().unwrap();

    dp.RCC.ahb2enr().modify(|_, w| w.gpioaen().set_bit());

    
    let mut rcc = dp.RCC.constrain();
    let mut gpioa = dp.GPIOA.split(&mut rcc);

    let mut led = gpioa.pa5.into_push_pull_output();

    // Create a delay provider using SysTick timer
    let mut delay = Delay::new(cp.SYST, 16_000_000);

    loop {
                // Turn LED on (set pin high)
        led.set_high();
        
        // Wait for 500 milliseconds
        delay.delay_ms(500u32);
        
        // Turn LED off (set pin low)
        led.set_low();
        
        // Wait for 500 milliseconds
        delay.delay_ms(500u32);
    }
}
