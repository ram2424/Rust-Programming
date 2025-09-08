#![no_main]
#![no_std]

use panic_halt as _;

use hal::prelude::*;
use hal::pwr::PwrExt;
use hal::rcc::Config;
use hal::stm32;
use hal::time::ExtU32;
use hal::timer::Timer;
use stm32g4xx_hal as hal;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    // 1. Get access to the core and device peripherals
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // 2. Configure the clock system
    let pwr = dp.PWR.constrain().freeze();
    let mut rcc = dp.RCC.freeze(Config::hsi(), pwr);

    // Configure clocks - use the correct API
    let mut delay_syst = cp.SYST.delay(&rcc.clocks);

    // 3. Configure the GPIO pin for the LED
    let mut gpioa = dp.GPIOA.split(&mut rcc);
    let mut led = gpioa.pa5.into_push_pull_output();

    // 4. Configure Timer2 for delay
    let timer2 = Timer::new(dp.TIM2, &rcc.clocks);
	let mut delay_tim2 = timer2.start_count_down(100.millis()).delay();

    // 5. Loop forever, blinking the LED
    loop {
        led.toggle();
        delay_syst.delay(1000.millis());
        led.toggle();
        delay_tim2.delay_ms(1000);
    }
}