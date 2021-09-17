# z80-mbc2-emu -- Z80-MBC2 emulator

## What is this?

This is an emulator to run the ROM and disk images prepared for the Just4Fun's Z80-MBC2. The Z80-MBC2 is an easy to build Z80 single board computer with just 4 ICs. See [Z80-MBC2: a 4 ICs homebrew Z80 computer](https://hackaday.io/project/159973-z80-mbc2-a-4-ics-homebrew-z80-computer). It can be used to test the SD content on a Linux, MacOS or Windows computer.

## What it does?

The emulator is based on the `S220718-R240620` version of the Z80-MBC2 firmware. It can run `Forth`, `CP/M 2.2`, `QP/M 2.71`, `CP/M 3.0`, `UCSD Pascal` and `Collapse OS`.


## Usage

The Z80-MBC2 SD contents must be extraced to a directory named `sd`. Use the zip file in https://cdn.hackaday.io/files/1599736844284832/SD-S220718-R240620-v1.zip .

To know the boot options available execute `z80-mbc2-emu` without parameters:
```
$ ./z80-mbc2-emu 
Usage: z80-mbc2-emu IMAGE
  IMAGE can be:

    forth for Forth using sd/forth13.bin
    autoboot for Autoboot using sd/autoboot.bin
    cpm22 for CP/M 2.2 using sd/cpm22.bin
    qpm for QP/M 2.71 using sd/QPMLDR.BIN
    cpm3 for CP/M 3.0 using sd/CPMLDR.COM
    pascal for UCSD Pascal using sd/ucsdldr.bin
    collapse for Collapse OS using sd/cos.bin

Download the images from https://cdn.hackaday.io/files/1599736844284832/S220718-R240620_IOS-Z80-MBC2.zip into the 'sd' directory.
```

To boot on any of the available environments execute `z80_mbc2_emu` with the code of the environment. For example:
```
$ ./z80-mbc2-emu cpm22
z80-mbc2-emu https://github.com/ivanizag/iz-cpm
Emulation of the Z80-MBC2, https://hackaday.io/project/159973

Press ctrl-c to return to host


Z80-MBC2 CP/M 2.2 BIOS - S030818-R140319
CP/M 2.2 Copyright 1979 (c) by Digital Research

A>DIR
A: ASCIART  BAS : ASM      COM : AUTOEXEC SUB : AUTOEXEC TXT
A: D        COM : DDT      COM : DUMP     COM : ED       COM
A: GENHEX   COM : GPELED   BAS : GPIO     BAS : HELLO    ASM
A: HELLO    COM : LOAD     COM : MAC      COM : MBASIC   COM
A: MBASIC85 COM : PEG      COM : PIP      COM : RTC      BAS
A: STARTREK BAS : STAT     COM : SUBMIT   COM : TREKINST BAS
A: USERLED  BAS : XMODEM   CFG : XMODEM   COM : XSUB     COM
A: ZDE16    COM : ZDENST16 COM
A>
```

Press control-c to exit the emulation.

## How does it work?

The Z80-MBC2 has a clever design based on a Z80 and a memory IC, both controlled by an Atmega microcontroller. The Atmega is able to put info on the data bus and can inject content to the RAM IC by generating code on the fly. It can also respond to IN and OUT ports with 1 bit adressing. It uses that as the interface with the Z80 programs. Via this interface it provides services related with the serial port, the SD card storage, the real time clock, the user led and button, and the GPIO.

This emulator emulates the Z80 and provides the same services given by the Atmega using the same IN and OUT ports. Instead of the serial port it uses the terminal. Instead of the SD it uses a directory named `sd` with the same contents.

## TODO

- Support interrupt driven serial transmission to support the Basic image
- Change the way to exit to host to allow control-c to be used on the emulation.
- Real time clock services
- GPIO services
- User led and button services
- Test in Windows and MacOS
