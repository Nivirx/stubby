#![no_std]
#![no_main]
#![allow(unused)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;

// Keep this line to ensure the `mem*` functions are linked in.
extern crate rlibc;

extern crate goblin;

extern crate uefi;
extern crate uefi_services;

#[macro_use]
extern crate uefi_macros;

use core::ffi::c_void;
use core::ptr;
use uefi::Handle;
use uefi::{prelude::*, Identify};
use uefi::{
    proto::console::gop::{BltOp, BltPixel, FrameBuffer, GraphicsOutput, PixelFormat},
    table::boot::{
        AllocateType, BootServices, MemoryType, OpenProtocolAttributes, OpenProtocolParams,
    },
};
use uefi_fb::UefiFb;

mod uefi_fb;
mod sync;

#[uefi_macros::entry]
pub fn efi_main(_image: Handle, mut st: SystemTable<Boot>) -> Status {
    // Initialize logging, memory allocation and uefi services
    let mut bt = match uefi_services::init(&mut st) {
        Ok(_) => {

            st.stdout().clear();
            st.stdout().set_color(
                uefi::proto::console::text::Color::LightMagenta,
                uefi::proto::console::text::Color::Black,
            );
            
            // output firmware-vendor (CStr16 to Rust string)
            // max size of 32 characters
            let mut buf = arrayvec::ArrayString::<32>::new();
            st.firmware_vendor().as_str_in_buf(&mut buf).unwrap();
            info!("Firmware Vendor: {}", buf.as_str());
        
            let rev = st.uefi_revision();
            let (major, minor) = (rev.major(), rev.minor());
            let buf = format!("UEFI {}.{}", major, minor / 10);
            info!("{}", buf);
        
            assert!(major >= 2, "Running on an old, unsupported version of UEFI");
            assert!(
                minor >= 30,
                "Old version of UEFI 2, some features might not be available."
            );

            st.boot_services()
        }
        Err(_) => {
            panic!();
        }
    };

    let mut fb = uefi_fb::UefiFb::new();
    
    // None sets a sane default of 1024x768
    fb.set_graphics_mode(None);
    fb.fb_blt_fill(BltPixel::new(100, 149, 237), (0, 0), (1024, 768));

    let mi = fb.current_mode_info();
    let stride = mi.stride();
    let (width, height) = mi.resolution();

    let mut fill_rectangle = |(x1, y1), (x2, y2), color| {
        assert!((x1 < width) && (x2 < width), "Bad X coordinate");
        assert!((y1 < height) && (y2 < height), "Bad Y coordinate");
        for row in y1..y2 {
            for column in x1..x2 {
                unsafe {
                    let pixel_index = (row * stride) + column;
                    let pixel_base = 4 * pixel_index;
                    fb.draw_fb(pixel_base, color);
                }
            }
        }
    };

    fill_rectangle((50, 30), (150, 600), [250, 128, 64]);
    fill_rectangle((400, 120), (750, 450), [16, 128, 255]);

    info!("GOP Framebuffer test complete");
    Status::SUCCESS
}