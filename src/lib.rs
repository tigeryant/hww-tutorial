#![no_std]
#![no_main]

pub mod init;

use nb::block;

// Simple protocol constants
const START_MARKER: u8 = 0xF0;
const END_MARKER: u8 = 0xF1;
const ESCAPE_CHAR: u8 = 0xF2;

/// Receive a transaction from the host
/// Returns the number of bytes received
pub fn receive_transaction(
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
pub fn u16_to_ascii(value: u16) -> [u8; 5] {
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
