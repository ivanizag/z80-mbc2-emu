use std::fs;
use std::io;
use std::io::Seek;
use std::io::Read;
use std::io::Write;

const TRACKS: u16 = 512;
const SECTORS: u8 = 32;
const SECTOR_SIZE: u16 = 512;

#[derive(Clone, Copy, PartialEq, Debug)]
enum FsError {
    // Error codes from Petit FatFs
    Ok = 0,
    DiskError = 1,
    //NotReady = 2,
    NoFile = 3,
    NotOpened = 4,
    //NotEnabled = 5,
    //NoFilesystem = 6,

    IllegalDiskNumber = 16,
    IllegalTrackNumber = 17,
    IllegalSectorNumber = 18,
}

pub struct FileSystem {
    file: Option<fs::File>,
    track: u16,
    sector: u8,
    last_error: FsError,
}

impl FileSystem  {
    pub fn new() -> FileSystem {
        FileSystem {
            file: None,
            track: 0,
            sector: 0,
            last_error: FsError::Ok,
        }
    }

    pub fn get_last_error(&self) -> u8 {
        self.last_error as u8
    }

    pub fn select_disk(&mut self, disk_set: u8, disk_number: u8) {
        let filename = format!("sd/DS{}N{:02}.DSK", disk_set, disk_number);

        if disk_set > 9 || disk_number > 99 {
            self.last_error = FsError::IllegalDiskNumber
        } else {
            let result = fs::OpenOptions::new()
                .write(true)
                .read(true)
                .open(&filename);

            self.last_error = match result {
                Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                    FsError::NoFile
                },
                Err(_) => {
                    FsError::DiskError
                },
                Ok(file) => {
                    self.file = Some(file);
                    FsError::Ok
                }
            }
        }
    }

    pub fn select_track(&mut self, track: u16) {
        if track < TRACKS {
            self.track = track;
            self.last_error = FsError::Ok;
        } else {
            self.last_error = FsError::IllegalTrackNumber;
        }
    }

    pub fn select_sector(&mut self, sector: u8) {
        if sector < SECTORS {
            self.sector = sector;
            self.last_error = FsError::Ok;
        } else {
            self.last_error = FsError::IllegalSectorNumber;
        }
    }

    fn sector_pos(&mut self) -> u64 {
        ((self.track as u64) * (SECTORS as u64) + (self.sector as u64)) *
        (SECTOR_SIZE as u64)
    }

    pub fn seek(&mut self) {
        let pos = self.sector_pos();

        self.last_error = match self.file.as_mut() {
            None => FsError::NotOpened,
            Some(f) => {
                match f.seek(io::SeekFrom::Start(pos)) { 
                    Err(_) => FsError::DiskError,
                    Ok(_) => FsError::Ok,
                }
            }
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.last_error != FsError::Ok {
            return 0
        }

        let mut value: u8 = 0;
        self.last_error = match self.file.as_mut() {
            None => FsError::NotOpened,
            Some(f) => {
                let mut buffer: [u8; 1] = [0; 1];
                match f.read(&mut buffer) {
                    Err(_) => FsError::DiskError,
                    Ok(_) => {
                        value = buffer[0];
                        FsError::Ok
                    }
                }
            }
        };
        value
    }

    pub fn write(&mut self, data: u8) {
        if self.last_error != FsError::Ok {
            return
        }
        self.last_error = match self.file.as_mut() {
            None => FsError::NotOpened,
            Some(f) => {
                let mut buffer: [u8; 1] = [0; 1];
                buffer[0] = data;
                match f.write(&mut buffer) {
                    Err(_) => FsError::DiskError,
                    Ok(_) => FsError::Ok
                }
            }
        };



    }

}
