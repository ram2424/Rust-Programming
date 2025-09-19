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
    info!("Starting CAN RX example...");
    
    info!("Initializing board & clocks...");
    // Configure clocks to use PCLK1 for FDCAN (avoiding HSE dependency)
    let mut config = embassy_stm32::Config::default();
    
    // Configure FDCAN to use PCLK1 instead of HSE
    config.rcc.mux.fdcansel = embassy_stm32::rcc::mux::Fdcansel::PCLK1;
    
    let p = embassy_stm32::init(config);
    info!("Board initialized successfully");
    
    info!("Configuring CAN1 on PA11 (RX) / PA12 (TX)...");
    let mut can_cfg = can::CanConfigurator::new(p.FDCAN1, p.PA11, p.PA12, Irqs);
    info!("CAN configurator created");
    
    // Accept all standard IDs into FIFO0
    info!("Setting up standard filter...");
    can_cfg.properties().set_standard_filter(
        can::filter::StandardFilterSlot::_0,
        can::filter::StandardFilter::accept_all_into_fifo0(),
    );
    info!("Standard filter configured");
    
    // Set CAN bitrate
    info!("Setting CAN bitrate to 250kbps...");
    can_cfg.set_bitrate(250_000);
    info!("Bitrate set");
    
    // Start CAN in internal loopback mode
    info!("Starting CAN in internal loopback mode...");
    let mut can = can_cfg.start(can::OperatingMode::NormalOperationMode);
    info!("CAN started successfully");
    
    Timer::after(Duration::from_millis(10)).await;
    info!("CAN initialized, sending test frame...");
    
    // Create a test frame (standard ID 0x123, 8 bytes)
    let frame = match can::Frame::new_data(
        StandardId::new(0x123).unwrap(),
        &[0xDE, 0xAD, 0xBE, 0xEF, 0, 0, 0, 0],
    ) {
        Ok(f) => {
            info!("Test frame created successfully");
            f
        },
        Err(e) => {
            warn!("Failed to create CAN frame: {:?}", e);
            loop { Timer::after(Duration::from_millis(1000)).await; }
        }
    };
    
    // Send the frame
    info!("Attempting to send test frame...");
    match can.write(&frame).await {
        Some(_) => info!("Test frame sent successfully"),
        None => warn!("Frame not sent (FIFO full?)"),
    }
    
    // Setup LED on PA5
    info!("Setting up LED on PA5...");
    let mut led = Output::new(p.PA5, Level::Low, Speed::Low);
    info!("LED configured, entering receive loop...");
    
    let mut frame_counter = 0u16;
    
    loop {
        // Send a new frame every loop iteration
        frame_counter = frame_counter.wrapping_add(1);
        let frame = match can::Frame::new_data(
            StandardId::new(0x123).unwrap(),
            &[
                (frame_counter >> 8) as u8,  // High byte of counter
                (frame_counter & 0xFF) as u8, // Low byte of counter
                0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE
            ],
        ) {
            Ok(f) => f,
            Err(_) => {
                warn!("Failed to create frame in loop");
                Timer::after(Duration::from_millis(100)).await;
                continue;
            }
        };
        
        // Try to send the frame (non-blocking)
        if let Some(_) = can.write(&frame).await {
            info!("Sent frame #{}", frame_counter);
        }
        
        // Try to read frames (with timeout using try_read in a loop)
        let mut received_frame = false;
        for _ in 0..10 { // Try reading for a short time
            if let Ok(env) = can.read().await {
                let frame = env.frame;
                // Extract CAN ID and data
                let id = match frame.header().id() {
                    Id::Standard(id) => id.as_raw() as u32,
                    Id::Extended(id) => id.as_raw(),
                };
                let len = frame.header().len();
                let data = &frame.data()[0..len as usize];
                info!("RX frame: id=0x{:X}, len={}, data={:?}", id, len, data);
                
                received_frame = true;
                break;
            }
            Timer::after(Duration::from_millis(1)).await;
        }
        
        // Blink LED if we received a frame
        if received_frame {
            info!("Blinking LED...");
            led.set_high();
            Timer::after(Duration::from_millis(200)).await;
            led.set_low();
            Timer::after(Duration::from_millis(200)).await;
        } else {
            info!("No frame received this cycle");
            Timer::after(Duration::from_millis(100)).await;
        }
        
        // Wait before next iteration
        Timer::after(Duration::from_millis(500)).await;
    }
}