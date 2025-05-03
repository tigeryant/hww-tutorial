#![no_std]
#![no_main]

// Ensure we halt the program on panic
use panic_halt as _;

// Alias for our HAL crate
use rp235x_hal as hal;

use hal::clocks::Clock;
use hal::fugit::RateExtU32;
use hal::uart::{DataBits, StopBits, UartConfig};
use nb::block;
// use embedded_hal_nb::serial::Read;

/// Tell the Boot ROM about our application
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

/// External high-speed crystal on the Raspberry Pi Pico 2 board is 12 MHz.
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

// Simple protocol constants
const START_MARKER: u8 = 0xF0;
const END_MARKER: u8 = 0xF1;
const ESCAPE_CHAR: u8 = 0xF2;
const ACK: u8 = 0x06;  // Acknowledge receipt

#[hal::entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = hal::pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let uart0_pins = (
        // UART TX (characters sent from rp235x) on pin 4 (GPIO2) in Aux mode
        pins.gpio2.into_function(),
        // UART RX (characters received by rp235x) on pin 5 (GPIO3) in Aux mode
        pins.gpio3.into_function(),
    );
    let mut uart0 = hal::uart::UartPeripheral::new(pac.UART0, uart0_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(115200.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    uart0.write_full_blocking(b"Bitcoin Hardware Wallet\r\n");
    uart0.write_full_blocking(b"Ready to receive transactions\r\n");

    // Buffer to store received transaction
    let mut buffer = [0u8; 1024]; // Adjust size based on expected transaction size

    loop {
        // Wait for and receive a transaction
        let received_len = receive_transaction(&mut uart0, &mut buffer);
        
        // Process the transaction (in a real implementation)
        // For now, just acknowledge receipt
        uart0.write_full_blocking(b"Transaction received (");
        
        // Convert the length to ASCII and send it
        let len_str = u16_to_ascii(received_len as u16);
        uart0.write_full_blocking(&len_str);
        
        uart0.write_full_blocking(b" bytes)\r\n");
        
        // Send acknowledgment byte
        uart0.write_full_blocking(&[ACK]);
    }
}

/// Receive a transaction from the host
/// Returns the number of bytes received
fn receive_transaction(
    uart: &mut impl embedded_hal_nb::serial::Read<u8>,
    buffer: &mut [u8]
) -> usize {
    let mut idx = 0;
    let mut escape_next = false;
    
    // Wait for start marker
    loop {
        let byte = match block!(uart.read()) {
            Ok(b) => b,
            Err(_) => continue, // Handle error by continuing to try reading
        };
        if byte == START_MARKER {
            break;
        }
    }
    
    // Read data until end marker
    loop {
        let byte = match block!(uart.read()) {
            Ok(b) => b,
            Err(_) => continue, // Handle error by continuing to try reading
        };
        
        if escape_next {
            // This byte was escaped, add it literally
            if idx < buffer.len() {
                buffer[idx] = byte;
                idx += 1;
            }
            escape_next = false;
        } else if byte == ESCAPE_CHAR {
            // Next byte is escaped
            escape_next = true;
        } else if byte == END_MARKER {
            // End of transaction
            break;
        } else {
            // Normal byte
            if idx < buffer.len() {
                buffer[idx] = byte;
                idx += 1;
            }
        }
    }
    
    idx // Return the number of bytes received
}


/// Convert a u16 to ASCII bytes
fn u16_to_ascii(value: u16) -> [u8; 5] {
    let mut result = [b'0'; 5];
    let mut val = value;
    let mut i = 4;
    
    while val > 0 || i == 4 {
        result[i] = b'0' + (val % 10) as u8;
        val /= 10;
        if i == 0 {
            break;
        }
        i -= 1;
    }
    
    result
}

/// Program metadata for `picotool info`
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"Bitcoin Hardware Wallet"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];
