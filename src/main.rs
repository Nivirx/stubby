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


    

    let handle = bt.get_handle_for_protocol::<GraphicsOutput>()
        .expect("missing GraphicsOutput protocol");

    
    let mut gop = unsafe {
        let mut gop_proto = st.boot_services().open_protocol::<GraphicsOutput>(
            OpenProtocolParams {
                handle,
                agent: st.boot_services().image_handle(),
                controller: None,
            },
            // For this test, don't open in exclusive mode. That
            // would break the connection between stdout and the
            // video console.
            OpenProtocolAttributes::GetProtocol,
        )
        .expect("failed to open Graphics Output Protocol");

        gop_proto
    };
    
    set_graphics_mode(&mut gop);
    fill_color(&mut gop);
    draw_fb(&mut gop);
    info!("GOP Framebuffer test complete");
    
    // Return success status
    Status::SUCCESS
}

// Set a larger graphics mode.
fn set_graphics_mode(gop: &mut GraphicsOutput) {
    // We know for sure QEMU has a 1024x768 mode.
    let mode = gop
        .modes()
        .find(|mode| {
            let info = mode.info();
            info.resolution() == (1024, 768)
        })
        .unwrap();

    gop.set_mode(&mode).expect("Failed to set graphics mode");
}

// Fill the screen with color.
fn fill_color(gop: &mut GraphicsOutput) {
    let op = BltOp::VideoFill {
        // Cornflower blue.
        color: BltPixel::new(100, 149, 237),
        dest: (0, 0),
        dims: (1024, 768),
    };

    gop.blt(op).expect("Failed to fill screen with color");
}

// Draw directly to the frame buffer.
fn draw_fb(gop: &mut GraphicsOutput) {
    // The `virtio-gpu-pci` graphics device we use on aarch64 doesn't
    // support `PixelFormat::BltOnly`.
    if cfg!(target_arch = "aarch64") {
        return;
    }

    let mi = gop.current_mode_info();
    let stride = mi.stride();
    let (width, height) = mi.resolution();

    let mut fb = gop.frame_buffer();

    type PixelWriter = unsafe fn(&mut FrameBuffer, usize, [u8; 3]);
    unsafe fn write_pixel_rgb(fb: &mut FrameBuffer, pixel_base: usize, rgb: [u8; 3]) {
        fb.write_value(pixel_base, rgb);
    }
    unsafe fn write_pixel_bgr(fb: &mut FrameBuffer, pixel_base: usize, rgb: [u8; 3]) {
        fb.write_value(pixel_base, [rgb[2], rgb[1], rgb[0]]);
    }
    let write_pixel: PixelWriter = match mi.pixel_format() {
        PixelFormat::Rgb => write_pixel_rgb,
        PixelFormat::Bgr => write_pixel_bgr,
        _ => {
            info!("This pixel format is not supported by the drawing demo");
            return;
        }
    };

    let mut fill_rectangle = |(x1, y1), (x2, y2), color| {
        assert!((x1 < width) && (x2 < width), "Bad X coordinate");
        assert!((y1 < height) && (y2 < height), "Bad Y coordinate");
        for row in y1..y2 {
            for column in x1..x2 {
                unsafe {
                    let pixel_index = (row * stride) + column;
                    let pixel_base = 4 * pixel_index;
                    write_pixel(&mut fb, pixel_base, color);
                }
            }
        }
    };

    fill_rectangle((50, 30), (150, 600), [250, 128, 64]);
    fill_rectangle((400, 120), (750, 450), [16, 128, 255]);
}
