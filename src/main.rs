use std::thread;
use std::time::Duration;

use iz80::*;

mod filesystem;
mod mbc2_machine;

#[cfg(windows)]
mod console_windows;
#[cfg(unix)]
mod console_unix;

use self::mbc2_machine::Mbc2Machine;

static BOOT_BASIC: &'static [u8] = include_bytes!("../sd/basic47.bin");
static BOOT_FORTH: &'static [u8] = include_bytes!("../sd/forth13.bin");
static BOOT_CPM22: &'static [u8] = include_bytes!("../sd/cpm22.bin");

fn main() {

    // Init device
    let mut machine = Mbc2Machine::new();
    let mut cpu = Cpu::new_z80();

    // Select boot code
    let binary: &[u8];
    let binary_address: u16;
    let boot = "CPM22";
    match boot {
        "BASIC" => {
            // Uses interruptions for I/O, not supported
            binary = BOOT_BASIC;
            binary_address = 0;
        },
        "FORTH" => {
            binary = BOOT_FORTH;
            binary_address = 0x0100;
        },
        "CPM22" => {
            binary = BOOT_CPM22;
            binary_address = 0xD200 - 32;
        },
        _ => {
            panic!("boot mode not supported")
        }
    }

    // Load the code in memory
    for i in 0..binary.len() {
        machine.poke(binary_address + i as u16, binary[i]);
    }

    cpu.registers().set_pc(binary_address);
    cpu.set_trace(false);

    let mut n = 0;
    while !machine.quit {
        cpu.execute_instruction(&mut machine);

        if cpu.is_halted() {
            println!("HALT instruction");
            break;
        }

        if true /*slow*/ {
            n += 1;
            if n > 20 {
                thread::sleep(Duration::from_nanos(1000));
                n = 0;
            }
        }
    }
}
