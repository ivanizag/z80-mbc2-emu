use chrono::{DateTime, Local, Datelike, Timelike};

use iz80::Machine;

use super::filesystem::FileSystem;

#[cfg(windows)]
use super::console_windows::Console;
#[cfg(unix)]
use super::console_unix::Console;

const RAM_SIZE: usize = 128*1024;

const OPCODE_NOP: u8 = 0xff;

const INT_RX_MASK: u8 = 1;
const INT_SYS_TICK_MASK: u8 = 2;

pub struct Mbc2Machine {
    mem: [u8; RAM_SIZE],
    disk_set: u8,

    bank: u8,
    opcode: u8,
    last_rx_is_empty: bool,
    io_byte_count: u32,
    track_sel_lo: u8,
    last_time: DateTime<Local>,
    pub quit: bool,

    con: Console,
    fs: FileSystem, 

    user_led: bool,
    gpio_a: u8,
    gpio_b: u8,
    io_dir_a: u8,
    io_dir_b: u8,
    ggpu_a: u8,
    ggpu_b: u8,

    pub int_raised: bool,
    int_status: u8,
    rx_done: bool,
    pub int_rx: bool,
    pub int_sys_tick: bool,
    sys_tick_time: u8,

    cpm_warm_boot: bool,
    spp: bool,
    spp_fd: bool,

    pub trace: bool,
}

impl Mbc2Machine {
    pub fn new() -> Mbc2Machine {
        Mbc2Machine {
            mem: [0; RAM_SIZE],
            disk_set: 0xff,
            bank: 0,
            opcode: OPCODE_NOP,
            last_rx_is_empty: false,
            io_byte_count: 0,
            track_sel_lo: 0,
            last_time: Local::now(),
            quit: false,

            con: Console::new(),
            fs: FileSystem::new(),

            user_led: false,
            gpio_a: 0,
            gpio_b: 0,
            io_dir_a: 0,
            io_dir_b: 0,
            ggpu_a: 0,
            ggpu_b: 0,

            int_raised: false,
            int_status: 0,
            rx_done: true,
            int_rx: false,
            int_sys_tick: false,
            sys_tick_time: 0,

            cpm_warm_boot: false,
            spp: false,
            spp_fd: false,
        
            trace: false,
        }
    }

    pub fn set_disk_set(&mut self, disk_set: u8) {
        self.disk_set = disk_set;
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

    pub fn tick_ms(&mut self) {
        if self.int_sys_tick {
            // TODO
        }
        if self.int_rx && self.con.status() && self.rx_done {
            self.int_status = self.int_status | INT_RX_MASK;
            self.int_raised = true;
            self.rx_done = false;
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
            let mut implemented = true;
            match self.opcode {
                0x00 => self.user_led = value & 1 != 0, // USER LED
                0x01 => self.con.put(value), // SERIAL TX
                0x03 => self.gpio_a = value, // GPIOA WRITE
                0x04 => self.gpio_b = value, // GPIOB WRITE
                0x05 => self.io_dir_a = value, // IODIRA WRITE
                0x06 => self.io_dir_b = value, // IODIRB WRITE
                0x07 => self.ggpu_a = value, // GGPUAA WRITE
                0x08 => self.ggpu_b = value, // GGPUAB WRITE
                0x09 => self.fs.select_disk(self.disk_set, value), // SELDISK
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
                0x0b => self.fs.select_sector(value), // SELSECT
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
                0x0e => { // SETIRQ
                    //   I/O DATA:    D7 D6 D5 D4 D3 D2 D1 D0
                    //               ---------------------------------------------------------
                    //                 X  X  X  X  X  X  X  0    Serial Rx IRQ not enabled
                    //                 X  X  X  X  X  X  X  1    Serial Rx IRQ enabled
                    //                 X  X  X  X  X  X  0  X    Systick IRQ not enabled
                    //                 X  X  X  X  X  X  1  X    Systick IRQ enabled
                    self.int_rx = value & INT_RX_MASK != 0;
                    self.int_sys_tick = value & INT_SYS_TICK_MASK != 0;
                },
                0x0f => { // SETTICK
                    if value > 0 {
                        self.sys_tick_time = value;
                    }
                },
                0x10 => { // SETOPT
                    self.cpm_warm_boot = value & 1 != 0;
                },
                0x11 => { // SETSPP
                    self.spp = true;
                    self.spp_fd = value & 1 != 0;
                },
                //0x12 => { // WRSPP
                //    // Todo: write value to a printer.out file.
                //},
                _ => implemented = false,
            }

            if !implemented {
                println!("<<{} not implemented>>",
                    opcode_name(self.opcode));
                self.quit = true;
            } else if self.trace
                    && self.opcode != OPCODE_NOP
                    && self.opcode != 0x01
                    && self.opcode != 0x0d
                    && (self.opcode != 0x0c || self.io_byte_count == 1) {
                println!("<<{}({:02x}) -> {}>>",
                    opcode_name(self.opcode), value, self.fs.get_last_error());
            }

            if self.opcode != 0x0a && self.opcode != 0x0c {
                // All done for the single byte opcodes
                self.opcode = OPCODE_NOP;
            }

        }
    }

    fn port_in(&mut self, address: u16) -> u8 {
        let a0 = (address & 1) == 1;
        if a0 {
            // Serial reception

            // NOTE 1: If there is no input char, a value 0xFF is forced as input char.
            // NOTE 2: The INT_ signal is always reset (set to HIGH) after this I/O operation.
            // NOTE 3: This is the only I/O that do not require any previous STORE OPCODE operation (for fast polling).
            // NOTE 4: A "RX buffer empty" flag and a "Last Rx char was empty" flag are available in the SYSFLAG opcode 
            //         to allow 8 bit I/O.
            self.int_status = self.int_status & !INT_RX_MASK; // Reset the RX signal
            self.int_raised = false;
            self.rx_done = true;

            if self.con.status() {
                let mut ch = self.con.read();
                if ch == 3 { // Control C
                    self.quit = true;
                } else if ch == 127 { // Backspace
                    ch = 8
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
            let mut implemented = true;
            let value = match self.opcode {
                0x80 => 0, /* not pressed */ // USER KEY
                0x81 => self.gpio_a, // GPIOA READ
                0x82 => self.gpio_b, // GPIOB READ
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
                    //                   X  X  X  0  X  X  X  X    CP/M warm boot message disabled
                    //                   X  X  X  1  X  X  X  X    CP/M warm boot message enabled
                    //
                    // NOTE: Currently only D0-D4 are used
                    let mut sysflags: u8 = 0b0010;
                    if self.con.status() {
                        sysflags += 0b0100;
                    }
                    if self.last_rx_is_empty {
                        sysflags += 0b1000;
                    }
                    if self.cpm_warm_boot {
                        sysflags += 0b1_0000;
                    }
                    sysflags
                },
                0x84 => {
                    if self.io_byte_count == 0 {
                        self.last_time = Local::now();
                    }
                    let value = match self.io_byte_count {
                        0 => self.last_time.second() as u8,
                        1 => self.last_time.minute() as u8,
                        2 => self.last_time.hour() as u8,
                        3 => self.last_time.day() as u8,
                        4 => self.last_time.month() as u8,
                        5 => (self.last_time.year() % 100) as u8,
                        6 => 21, // 21º Celsius
                        _ => 0,
                    };
                    self.io_byte_count += 1;
                    if self.io_byte_count >= 7 {
                        self.opcode = OPCODE_NOP;
                    }
                    value
                },
                0x85 => self.fs.get_last_error(), // ERRDISK
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
                0x87 => 0, //SDMOUNT
                0x88 => 255, // ATXBUFF, buffers are never full
                0x89 => { // SYSIRQ
                    //    I/O DATA:    D7 D6 D5 D4 D3 D2 D1 D0
                    //                ---------------------------------------------------------
                    //                  X  X  X  X  X  X  X  0    Serial Rx IRQ not set
                    //                  X  X  X  X  X  X  X  1    Serial Rx IRQ set
                    //                  X  X  X  X  X  X  0  X    Systick IRQ not set
                    //                  X  X  X  X  X  X  1  X    Systick IRQ set
                    let value = self.int_status;
                    self.int_status = 0;
                    value
                },
                0x90 => { // GETSPP
                    //    I/O DATA:  D7 D6 D5 D4 D3 D2 D1 D0
                    //              ---------------------------------------------------------
                    //                0  0  0  0  0  0  0  0    SPP emulation disabled
                    //               D7 D6 D5 D4 D3  0  0  1    SPP emulation enabled
                    //
                    //     bit  | SPP Status line
                    //    ----------------------------------
                    //     D0  | 1 (SPP emulation enabled)
                    //     D1  | 0 (not used)
                    //     D2  | 0 (not used)
                    //     D3  | ACK (active Low) -> 0
                    //     D4  | BUSY (active High) -> 0
                    //     D5  | PAPEREND (active High) -> 0
                    //     D6  | SELECT (active High) -> 1
                    //     D7  | ERROR (active Low) -> 0
                    if self.spp {
                        0b0100_0001 // The pseudo printer is always ready
                    } else {
                        0
                    }
                },
                _ => {
                    implemented = false;
                    0
                }
            };
            if !implemented {
                println!("<<{} not implemented>>",
                    opcode_name(self.opcode));
                self.quit = true;
            } else if self.trace
                    && self.opcode != OPCODE_NOP
                    && self.opcode != 0x83
                    && (self.opcode != 0x86 || self.io_byte_count == 1) {
                println!("<<{} -> {:02x}, {}>>",
                opcode_name(self.opcode), value, self.fs.get_last_error());
            }
            value
        }
    }
}

fn opcode_name(opcode: u8) -> &'static str {
    match opcode {
        0x00 => "USER LED",

        0x01 => "SERIAL TX",
        0x03 => "GPIOA W",
        0x04 => "GPIOB W",
        0x05 => "IODIRA W",
        0x06 => "IODIRB W",
        0x07 => "GPPUA W",
        0x08 => "GPPUB W",
        0x09 => "SELDISK",
        0x0A => "SELTRACK",
        0x0B => "SELSECT",
        0x0C => "WRITESECT",
        0x0D => "SETBANK",
        0x0E => "SETIRQ",
        0x0F => "SETTICK",
        0x10 => "SETOPT",
        0x11 => "SETSPP",
        0x12 => "WRSPP",

        0x80 => "USER KEY",
        0x81 => "GPIOA R",
        0x82 => "GPIOB R",
        0x83 => "SYSFLAGS",
        0x84 => "DATETIME",
        0x85 => "ERRDISK",
        0x86 => "READSECT",
        0x87 => "SDMOUNT",
        0x88 => "ATXBUFF",
        0x89 => "SYSIRQ",
        0x8A => "GETSPP",

        0xFF => "NOP",
        _ => "UNKNOWN"
    }
}
