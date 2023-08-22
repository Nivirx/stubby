# stubby

Stubby is a project that aims to explore and experiment with UEFI firmware and its features. It is based on another one of my projects called newt_stub.

## Features

- Stubby can boot from UEFI and print some basic information on the screen.
- Stubby can interact with UEFI protocols and services, such as memory allocation, file system access, and graphics output.
- Stubby can load and execute other EFI applications, such as the shell or the bootloader.

## Requirements

- Linux, macOS or WSL
- A Rust compiler and cargo toolchain.
- A QEMU emulator and OVMF firmware.
- A ***GNU*** Make[^note2] utility.

## Usage

To build and run stubby, use the following command:

```bash
make run-debug
```

This will compile the stubby binary, create a virtual disk image[^note1] with the EFI application, and launch QEMU with OVMF.

To clean up the generated files, use the following command:

```bash
make clean
```

## License

Stubby is licensed under the MIT license. See the LICENSE file for more details.

[^note1]: Currently binaries are dumped into a folder and QEMU 'mounts' that folder as a FAT filesystem
[^note2]: Please only use GNU Make, BSD Make (and by extention 'Apple Make') have issues with the Makefile
