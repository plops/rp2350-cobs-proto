#![no_std]
#![no_main]

extern crate alloc;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config as AdcConfig, InterruptHandler};
use embassy_rp::gpio::Pull;
use embassy_rp::uart::{Config as UartConfig, UartTx};
use embassy_rp::bind_interrupts;
use embassy_time::{Duration, Instant, Timer};
use prost::Message;
use {defmt_rtt as _, panic_probe as _};

use embedded_alloc::LlffHeap;

// 1. Initialize the global memory allocator.
// Prost requires the alloc crate to compile. Even if we do not perform dynamic allocations
// at runtime, the linker requires a global allocator to be registered.
#[global_allocator]
static HEAP: LlffHeap = LlffHeap::empty();

// Include the compiled protobuf modules
pub mod messages {
    include!(concat!(env!("OUT_DIR"), "/adc_monitor.rs"));
}

// 2. Bind the ADC interrupt handler.
bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the heap allocator (16 KB heap)
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 16384;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe {
            let ptr = core::ptr::addr_of_mut!(HEAP_MEM) as *mut u8;
            HEAP.init(ptr as usize, HEAP_SIZE);
        }
    }

    let p = embassy_rp::init(Default::default());
    info!("RP2350 Initialized!");

    // 3. Configure and Initialize the ADC.
    let mut adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    // We use PIN_26 (ADC0) as our analog input.
    let mut adc_pin = Channel::new_pin(p.PIN_26, Pull::None);

    // 4. Configure and Initialize UART0 TX.
    // Baudrate defaults to 115200. GPIO0 is the TX pin for UART0.
    let uart_config = UartConfig::default();
    let mut tx = UartTx::new_blocking(p.UART0, p.PIN_0, uart_config);

    let start_time = Instant::now();

    loop {
        // Read raw ADC value (12-bit, range 0..=4095)
        let raw_val = match adc.read(&mut adc_pin).await {
            Ok(val) => val,
            Err(e) => {
                error!("ADC Read failed: {:?}", defmt::Debug2Format(&e));
                Timer::after(Duration::from_millis(500)).await;
                continue;
            }
        };

        // Calculate voltage (assume 3.3V reference voltage)
        let voltage = (raw_val as f32 * 3.3) / 4095.0;
        let timestamp_ms = Instant::now().duration_since(start_time).as_millis() as u32;

        // 1. Construct the Protobuf message payload
        let reading = messages::AdcReading {
            timestamp_ms,
            adc_raw: raw_val as u32,
            voltage,
        };

        // 2. Serialize protobuf struct to raw bytes
        let mut proto_buf = [0u8; 32];
        let proto_len = reading.encoded_len();
        if proto_len > proto_buf.len() {
            error!("Protobuf message too large for buffer!");
            continue;
        }

        let mut write_buf = &mut proto_buf[..];
        if let Err(e) = reading.encode(&mut write_buf) {
            error!("Protobuf encoding failed: {:?}", defmt::Debug2Format(&e));
            continue;
        }

        // 3. Encode using COBS framing
        // COBS overhead requires max: N + N/254 + 1, plus 1 for delimiter
        let mut cobs_buf = [0u8; 36];
        if proto_len + 2 > cobs_buf.len() {
            error!("COBS destination buffer too small!");
            continue;
        }

        let cobs_payload_len = cobs::encode(&proto_buf[..proto_len], &mut cobs_buf[..34]);
        
        // Append the 0x00 delimiter to mark the end of the packet frame
        cobs_buf[cobs_payload_len] = 0x00;
        let total_tx_len = cobs_payload_len + 1;

        // 4. Transmit packet over UART TX
        match tx.blocking_write(&cobs_buf[..total_tx_len]) {
            Ok(_) => {
                info!("Sent ADC reading: raw={}, volt={}, time={}", raw_val, voltage, timestamp_ms);
            }
            Err(e) => {
                error!("UART TX failed: {:?}", defmt::Debug2Format(&e));
            }
        }

        // Sample every 250 ms
        Timer::after(Duration::from_millis(250)).await;
    }
}
