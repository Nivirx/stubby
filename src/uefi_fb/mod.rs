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
use crate::sync::Spinlock::*;

struct UefiFb {
    gop: Arc<ScopedProtocol<GraphicsOutput>>
}

impl UefiFb {

    pub fn new() -> UefiFb {
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
        
        
        UefiFb {
            gop: Arc::new(gop)
        }
        

        /*
        set_graphics_mode(&mut gop);
        fill_color(&mut gop);
        draw_fb(&mut gop);
        info!("GOP Framebuffer test complete");
        
        // Return success status
        Status::SUCCESS
        */
        }
    
    // Set a larger graphics mode.
    pub fn set_graphics_mode(&mut self) {
        // We know for sure QEMU has a 1024x768 mode.
        let mode = self.gop
            .modes()
            .find(|mode| {
                let info = mode.info();
                info.resolution() == (1024, 768)
            })
            .unwrap();

        self.gop.set_mode(&mode).expect("Failed to set graphics mode");
    }

    // Fill the screen with color.
    fn fill_color(&mut self) {
        let op = BltOp::VideoFill {
            // Cornflower blue.
            color: BltPixel::new(100, 149, 237),
            dest: (0, 0),
            dims: (1024, 768),
        };

        self.gop.blt(op).expect("Failed to fill screen with color");
    }

    // Draw directly to the frame buffer.
    fn draw_fb(&mut self) {
        // The `virtio-gpu-pci` graphics device we use on aarch64 doesn't
        // support `PixelFormat::BltOnly`.
        if cfg!(target_arch = "aarch64") {
            return;
        }

        let mi = self.gop.current_mode_info();
        let stride = mi.stride();
        let (width, height) = mi.resolution();

        let mut fb = self.gop.frame_buffer();

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
}