#![no_std]
#![no_main]
mod fmt;
#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::FDCAN1;
use embassy_stm32::{bind_interrupts, can};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::{Duration, Timer};
use embassy_stm32::time::Hertz;
use embedded_can::{StandardId, Id};

bind_interrupts!(struct Irqs {
    FDCAN1_IT0 => can::IT0InterruptHandler<FDCAN1>;
    FDCAN1_IT1 => can::IT1InterruptHandler<FDCAN1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting STM32 Simple Bidirectional CAN...");
    
    // Initialize hardware
    let mut config = embassy_stm32::Config::default();
    config.rcc.mux.fdcansel = embassy_stm32::rcc::mux::Fdcansel::PCLK1;
    let p = embassy_stm32::init(config);
    
    // Configure CAN
    let mut can_cfg = can::CanConfigurator::new(p.FDCAN1, p.PA11, p.PA12, Irqs);
    
    // Accept all standard IDs
    can_cfg.properties().set_standard_filter(
        can::filter::StandardFilterSlot::_0,
        can::filter::StandardFilter::accept_all_into_fifo0(),
    );
    
    can_cfg.set_bitrate(250_000);
    let mut can = can_cfg.start(can::OperatingMode::NormalOperationMode);
    
    // Status LED
    let mut led = Output::new(p.PA5, Level::Low, Speed::Low);
    
    info!("STM32 CAN ready - sending on 0x100, listening for 0x200");
    
    let mut counter = 0u16;
    let mut last_tx = embassy_time::Instant::now();
    
    loop {
        // Send frame every 1 second
        if embassy_time::Instant::now() - last_tx > Duration::from_millis(1000) {
            counter += 1;
            last_tx = embassy_time::Instant::now();
            
            let frame = can::Frame::new_data(
                StandardId::new(0x100).unwrap(),  // STM32 sends on 0x100
                &[
                    0xAA, 0xBB,                   // STM32 signature
                    (counter >> 8) as u8,        // Counter high
                    (counter & 0xFF) as u8,      // Counter low
                    0x11, 0x22, 0x33, 0x44       // Test data
                ],
            ).unwrap();
            
            if let Some(_) = can.write(&frame).await {
                info!("STM32 TX: counter={}, data=[AA BB {:02X} {:02X} 11 22 33 44]", 
                      counter, (counter >> 8) as u8, (counter & 0xFF) as u8);
            }
        }
        
        // Non-blocking check for Arduino frames
        Timer::after(Duration::from_millis(10)).await;
        
        // Try to read with timeout
        match embassy_time::with_timeout(Duration::from_millis(1), can.read()).await {
            Ok(Ok(env)) => {
                let frame = env.frame;
                let id = match frame.header().id() {
                    Id::Standard(id) => id.as_raw(),
                    _ => 0,
                };
                let len = frame.header().len();
                let data = &frame.data()[0..len as usize];
                
                info!("STM32 RX: ID=0x{:X}, data={:?}", id, data);
                
                // Blink LED on receive
                led.set_high();
                Timer::after(Duration::from_millis(100)).await;
                led.set_low();
            }
            Ok(Err(e)) => {
                warn!("CAN read error: {:?}", e);
            }
            Err(_) => {
                // Timeout - no frame received, continue
            }
        }
    }
}