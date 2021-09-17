use iz80::*;

mod filesystem;
mod images;
mod mbc2_machine;

#[cfg(windows)]
mod console_windows;
#[cfg(unix)]
mod console_unix;

use self::mbc2_machine::Mbc2Machine;
use self::images::*;

// Welcome message
const WELCOME: &'static str =
"z80-mbc2-emu https://github.com/ivanizag/iz-cpm
Emulation of the Z80-MBC2, https://hackaday.io/project/159973

Press ctrl-c to return to host";


fn main() {
    let image = select_image();

    // Init device
    let mut machine = Mbc2Machine::new();
    let mut cpu = Cpu::new_z80();

    // Load the image
    if !load_image(&mut machine, &image) {
        return;
    }

    machine.set_disk_set(image.disk_set);
    cpu.registers().set_pc(image.address);
    cpu.set_trace(false);

    // Start the cpu
    println!("{}", WELCOME);
    while !machine.quit {
        cpu.execute_instruction(&mut machine);

        if cpu.is_halted() {
            println!("HALT instruction");
            break;
        }
    }
}
