use core::ffi::c_void;
use core::ptr;
use uefi::Handle;
use uefi::proto::ProtocolPointer;
use uefi::table::boot::ScopedProtocol;
use uefi::{prelude::*, Identify};
use uefi::{
    proto::console::gop::{BltOp, BltPixel, FrameBuffer, GraphicsOutput, PixelFormat},
    table::boot::{
        AllocateType, BootServices, MemoryType, OpenProtocolAttributes, OpenProtocolParams,
    },
};
use alloc::sync::Arc;
use crate::sync;
use crate::sync::{Spinlock, SpinlockGuard};

pub struct UefiFb<'a> {
    gop: ScopedProtocol<'a, GraphicsOutput<'a>>
}

impl UefiFb<'_> {

    pub fn new() -> UefiFb<'static> {
        let st_ref = unsafe { uefi_services::system_table().as_ref() };
        let bt = st_ref.boot_services();

        let handle = bt.get_handle_for_protocol::<GraphicsOutput>()
        .expect("missing GraphicsOutput protocol");

    
        let mut gop = unsafe {
            let mut gop_proto = bt.open_protocol::<GraphicsOutput>(
                OpenProtocolParams {
                    handle,
                    agent: bt.image_handle(),
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
        
        UefiFb { gop }
        
        }
    
    // Set a larger graphics mode.
    pub fn set_graphics_mode(&mut self, mode_tuple: Option<(i16,i16,i16)>) {
        if mode_tuple == None {
            // We know for sure QEMU has a 1024x768 mode.
            let mode = self.gop
            .modes()
            .find(|mode| {
                let info = mode.info();
                info.resolution() == (1024, 768)
            })
            .unwrap();

            self.gop.set_mode(&mode).expect("Failed to set graphics mode");
        } else {
            unimplemented!()
        }
    }

    // Fill the screen with color.
    fn fb_blt_fill(&mut self, color: BltPixel, dest: (usize, usize), dims: (usize, usize)) {
        let op = BltOp::VideoFill {
            color,
            dest,
            dims
        };

        self.gop.blt(op).expect("Failed to fill screen with color");
    }


    // Draw directly to the frame buffer.
    fn draw_fb(&mut self, index: usize, rgb: [u8; 3]) {
        let mi = self.gop.current_mode_info();
        let mut fb = self.gop.frame_buffer();

        type PixelWriter = unsafe fn(&mut FrameBuffer, usize, [u8; 3]);

        unsafe fn write_pixel_rgb(fb: &mut FrameBuffer, index: usize, rgb: [u8; 3]) {
            fb.write_value(index, rgb);
        }
        unsafe fn write_pixel_bgr(fb: &mut FrameBuffer, index: usize, rgb: [u8; 3]) {
            fb.write_value(index, [rgb[2], rgb[1], rgb[0]]);
        }

        let write_pixel: PixelWriter = match mi.pixel_format() {
            PixelFormat::Rgb => write_pixel_rgb,
            PixelFormat::Bgr => write_pixel_bgr,
            _ => {
                info!("This pixel format is not supported by the drawing demo");
                return;
            }
        };

        unsafe { write_pixel(&mut fb, index, rgb) };
    }
}