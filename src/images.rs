use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process;

use iz80::Machine;

use super::mbc2_machine::Mbc2Machine;

pub struct ImageDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub file: &'static str,
    pub address: u16,
    pub disk_set: u8,
    pub ints: bool,
}

const IMAGES_FOLDER: &str = "sd";


static IMAGES: [ImageDefinition; 8] = [
    ImageDefinition {id: "basic", name: "Basic", file: "basic47.bin",
        address: 0x0000, disk_set: 0xff, ints: true},
    ImageDefinition {id: "forth", name: "Forth", file: "forth13.bin",
        address: 0x0100, disk_set: 0xff, ints: false},
    ImageDefinition {id: "autoboot", name: "Autoboot", file: "autoboot.bin",
        address: 0x0000, disk_set: 0xff, ints: false},
    ImageDefinition {id: "cpm22", name: "CP/M 2.2", file: "cpm22.bin",
        address: 0xD1E0, disk_set: 0, ints: false},
    ImageDefinition {id: "qpm", name: "QP/M 2.71", file: "QPMLDR.BIN",
        address: 0x0080, disk_set: 1, ints: false},
    ImageDefinition {id: "cpm3", name: "CP/M 3.0", file: "CPMLDR.COM",
        address: 0x0100, disk_set: 2, ints: false},
    ImageDefinition {id: "pascal", name: "UCSD Pascal", file: "ucsdldr.bin",
        address: 0x0000, disk_set: 3, ints: false},
    ImageDefinition {id: "collapse", name: "Collapse OS", file: "cos.bin",
        address: 0x0000, disk_set: 4, ints: false},
];

const USAGE: &'static str =
"Usage: z80-mbc2-emu IMAGE
  IMAGE can be:
";

const USAGE2: &'static str =
"
Download the images from https://cdn.hackaday.io/files/1599736844284832/S220718-R240620_IOS-Z80-MBC2.zip into the 'sd' directory.
";

pub fn select_image() -> &'static ImageDefinition {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        usage();
        process::exit(1);
    }
    let selection = &args[1];

    for i in 0..IMAGES.len() {
        if IMAGES[i].id == selection && !IMAGES[i].ints{
            return &IMAGES[i];
        }
    }

    println!("image '{}' not found.", selection);
    usage();
    process::exit(1);
}

pub fn usage() {
    println!("{}", USAGE);
    for i in 0..IMAGES.len() {
        let filename = Path::new(IMAGES_FOLDER).join(Path::new(IMAGES[i].file));
        if !IMAGES[i].ints {
            println!("    {} for {} using {}", IMAGES[i].id, IMAGES[i].name, filename.to_str().unwrap());
        }
    }
    println!("{}", USAGE2);
}

pub fn load_image(machine: &mut Mbc2Machine, image: &ImageDefinition) -> bool {
    let filename = Path::new(IMAGES_FOLDER).join(Path::new(image.file));
    let mut file = match fs::File::open(&filename) {
        Ok(file) => file,
        Err(error) => {
            println!("Error opening the file '{}': {}",
                filename.to_string_lossy(), error);
            return false;
        }
    };

    let mut buf = [0u8;65536];
    let size = match file.read(&mut buf) {
        Ok(size) => size,
        Err(error) => {
            println!("Error reading the file '{}': {}",
                filename.to_string_lossy(), error);
            return false;
        }
    };

    // Load the code in memory
    for i in 0..size {
        machine.poke(image.address + i as u16, buf[i]);
    }

    true
}
