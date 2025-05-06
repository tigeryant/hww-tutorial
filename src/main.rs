#![no_std]
#![no_main]

// Ensure we halt the program on panic
use panic_halt as _;

// Alias for our HAL crate
use rp235x_hal as hal;

use hal::clocks::Clock;
use hal::fugit::RateExtU32;
use hal::uart::{DataBits, StopBits, UartConfig};
use hww_tutorial::init::init;
use hww_tutorial::{receive_transaction, u16_to_ascii};

/// Tell the Boot ROM about our application
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

// Simple protocol constants
const ACK: u8 = 0x06;  // Acknowledge receipt

#[hal::entry]
fn main() -> ! {
    let (pins, uart0_peripheral, mut resets, clocks) = init();

    let uart0_pins = (
        // UART TX (characters sent from rp235x) on pin 4 (GPIO2) in Aux mode
        pins.gpio2.into_function(),
        // UART RX (characters received by rp235x) on pin 5 (GPIO3) in Aux mode
        pins.gpio3.into_function(),
    );
    let mut uart0 = hal::uart::UartPeripheral::new(uart0_peripheral, uart0_pins, &mut resets)
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
