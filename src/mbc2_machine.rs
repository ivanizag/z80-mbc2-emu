use iz80::Machine;

use super::filesystem::FileSystem;

#[cfg(windows)]
use super::console_windows::Console;
#[cfg(unix)]
use super::console_unix::Console;

const RAM_SIZE: usize = 128*1024;

const OPCODE_NOP: u8 = 0xff;

pub struct Mbc2Machine {
    mem: [u8; RAM_SIZE],

    bank: u8,
    opcode: u8,
    last_rx_is_empty: bool,
    io_byte_count: u32,
    track_sel_lo: u8,
    pub quit: bool,

    con: Console,
    fs: FileSystem, 
}

impl Mbc2Machine {
    pub fn new() -> Mbc2Machine {
        Mbc2Machine {
            mem: [0; RAM_SIZE],
            bank: 0,
            opcode: OPCODE_NOP,
            last_rx_is_empty: false,
            io_byte_count: 0,
            track_sel_lo: 0,
            quit: false,

            con: Console::new(),
            fs: FileSystem::new(),
        }
    }
    fn decode_address(&self, address: u16) -> usize {
        let a15 = (address & 0x8000) != 0;
        let base = (address & 0x7fff) as usize;
        if a15 {
            // Upper addresses, fixed from 0x0_8000 to 0x0_FFFF
            return address as usize
        } else {
            // Lower addresses
            return match self.bank {
                0 => base, //from 0x0_0000 to 0x0_7FFF
                1 => base + 0x1_0000, //from 0x1_0000 to 0x1_7FFF
                2 => base + 0x1_8000, //from 0x1_8000 to 0x1_FFFF
                _ => base, // Default to 0
            }
        }
    }
}

impl Machine for Mbc2Machine {
    fn peek(&self, address: u16) -> u8 {
        let ram_address = self.decode_address(address);
        //println!("$$$ {:05x}", ram_address);

        self.mem[ram_address]
    }

    fn poke(&mut self, address: u16, value: u8) {
        let ram_address = self.decode_address(address);
        //println!("$$$ {:05x} W", ram_address);

        self.mem[ram_address] = value;
    }

    fn port_out(&mut self, address: u16, value: u8) {
        //println!("OUT({:04x}, {:02x})", address, value);
        let a0 = (address & 1) == 1;
        if a0 {
            // Store opcode
            self.opcode = value;
            self.io_byte_count = 0;
        } else {
            match self.opcode {
                0x01 => { // SERIAL TX
                    self.con.put(value)
                },
                0x09 => { // SELDISK
                    self.fs.select_disk(0/*CPM22*/, value)
                }
                0x0a => { // SELTRACK
                    if self.io_byte_count == 0 {
                        self.track_sel_lo = value;
                        self.io_byte_count += 1
                    } else {
                        let track: u16 = ((value as u16) << 8) + self.track_sel_lo as u16;
                        self.fs.select_track(track);
                        self.opcode = OPCODE_NOP;
                    }
                }
                0x0b => { // SELSECT
                    self.fs.select_sector(value)
                }
                0x0c => { // WRITESECT
                    if self.io_byte_count == 0 {
                        self.fs.seek();
                    }

                    self.fs.write(value);
                    self.io_byte_count += 1;
                    if self.io_byte_count >= 512 {
                        self.opcode = OPCODE_NOP;
                    }
                }
                0x0d => { // SETBANK
                    if value <= 2 {
                        self.bank = value
                    }
                },
                _ => {
                    panic!("Not implemented out opcode {:02x},{}", self.opcode, value);
                }
            }
            if self.opcode != 0x0a && self.opcode != 0x0c {
                // All done for the single byte opcodes
                self.opcode = OPCODE_NOP;
            }
        }
    }

    fn port_in(&mut self, address: u16) -> u8 {
        let a0 = (address & 1) == 1;
        //println!("IN({:04x})", address);
        if a0 {
            // Serial reception

            // NOTE 1: If there is no input char, a value 0xFF is forced as input char.
            // NOTE 2: The INT_ signal is always reset (set to HIGH) after this I/O operation.
            // NOTE 3: This is the only I/O that do not require any previous STORE OPCODE operation (for fast polling).
            // NOTE 4: A "RX buffer empty" flag and a "Last Rx char was empty" flag are available in the SYSFLAG opcode 
            //         to allow 8 bit I/O.
            if self.con.status() {
                let ch = self.con.read();
                if ch == 3 {
                    self.quit = true;
                }
                self.last_rx_is_empty = false;
                ch
            } else {
                // No data available
                self.last_rx_is_empty = true;
                0xff
            }
        } else {
            // Execute opcode
            match self.opcode {
                0x83 => {
                    // SYSFLAGS (Various system flags for the OS):
                    //     I/O DATA:    D7 D6 D5 D4 D3 D2 D1 D0
                    //                 ---------------------------------------------------------
                    //                   X  X  X  X  X  X  X  0    AUTOEXEC not enabled
                    //                   X  X  X  X  X  X  X  1    AUTOEXEC enabled
                    //                   X  X  X  X  X  X  0  X    DS3231 RTC not found
                    //                   X  X  X  X  X  X  1  X    DS3231 RTC found
                    //                   X  X  X  X  X  0  X  X    Serial RX buffer empty
                    //                   X  X  X  X  X  1  X  X    Serial RX char available
                    //                   X  X  X  X  0  X  X  X    Previous RX char valid
                    //                   X  X  X  X  1  X  X  X    Previous RX char was a "buffer empty" flag
                    //
                    // NOTE: Currently only D0-D3 are used
                    let mut sysflags: u8 = 0;
                    if self.con.status() {
                        sysflags += 0b0100;
                    }
                    if self.last_rx_is_empty {
                        sysflags += 0b1000;
                    }
                    sysflags
                },
                0x85 => { // ERRDISK
                    self.fs.get_last_error()
                }
                0x86 => { // READSECT
                    if self.io_byte_count == 0 {
                        self.fs.seek();
                    }

                    let value = self.fs.read();
                    self.io_byte_count += 1;
                    if self.io_byte_count >= 512 {
                        self.opcode = OPCODE_NOP;
                    }
                    value
                }
                _ => {
                    panic!("Not implemented in opcode {:02x}", self.opcode);
                }
            }
        }

    }
}
