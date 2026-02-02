/*

QR code detection and decoding

Using the rxing, esp-idf, and esp32-camera-rs crates
On the Xiao esp32s3 sense w/ camera

*/

use embedded_graphics::{
    draw_target::DrawTarget,
    image::Image,
    prelude::Point,
    Drawable,
    {image::ImageRaw, pixelcolor::Rgb565, prelude::RgbColor},
};

use esp_idf_hal::{
    delay::Delay,
    delay::Ets,
    gpio::AnyIOPin,
    gpio::{Gpio21, Output, PinDriver},
    peripherals::Peripherals,
    spi::{SpiConfig, SpiDeviceDriver, SpiDriverConfig},
    units::FromValueType,
};
use mipidsi::{models::GC9A01, Builder};

// Blink count and frequency for signaling a scan
const QR_SCAN_LED_BLINK_COUNT: u8 = 1;
const QR_SCAN_LED_DELAY_MS: u32 = 500;

// Blink count and frequency for signaling a successful scan
const QR_FOUND_LED_BLINK_COUNT: u8 = 10;
const QR_FOUND_LED_DELAY_MS: u32 = 50;

// QR code scanning delay
const DETECT_INTERVAL_SECONDS: u32 = 3;

const DETECT_INTERVAL_MS: u32 = DETECT_INTERVAL_SECONDS * 1000;

// Constants for the qrcode scanner set to equal frame dimensions
// FRAME_WIDTH and FRAME_HEIGHT must match dimensions of FRAMESIZE
// (see esp-camera-rs for FRAMESIZE constants)
const FRAME_WIDTH: u32 = 240;
const FRAME_HEIGHT: u32 = 240;
const FRAMESIZE: esp_idf_sys::camera::framesize_t = esp_camera_rs::FRAMESIZE_240X240;

macro_rules! blink {
    ($pin: expr, $times: expr, $delay:tt ms) => {{
        for _ in 0..$times {
            let _ = led.set_low();
            Delay::delay_ms(&Delay::default(), delay_ms);
            let _ = led.set_high();
            Delay::delay_ms(&Delay::default(), delay_ms);
        }
    }};
}

// Convert a greyscale (Vec<u8>) image to a 565 (Vec<u16>) Image
fn grey_to_565(greyscale: Vec<u8>) -> Vec<u16> {
    greyscale
        .iter()
        .map(|&grey| {
            let r = (grey >> 3) as u16; // 8 - 3 : 5 bits
            let g = (grey >> 2) as u16; // 8 - 2 : 6 bits
            let b = (grey >> 3) as u16; // 8 - 3 : 5 bits
            (r << 11) | (g << 5) | b
            // u8 in xxxx xxxx
            // r = 3 >> : 000x xxxx (5 remain)
            // g = 3 >> : 00xx xxxx (6 remain)
            // b = 3 >> : 000x xxxx (5 remain)
            // *Losing the bottom 2-3 bits (integers 0-8)*
            //
            //  << 11   << 5  << 0
            //     \/     \/    \/
            // 00000 000000 00000
            // | (Or) combines. Each value is shifted. All zeros for the others for a given section
        })
        .collect()
}

fn main() {
    // Esp-idf setup
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Get esp32s3 peripherals
    let peripherals = Peripherals::take().unwrap();

    // Display pin configuration
    let cs = peripherals.pins.gpio2; // D1 on xiao
    let rst = peripherals.pins.gpio3; // D2 on xiao
    let dc = peripherals.pins.gpio4; // D3 on xiao
    let sclk = peripherals.pins.gpio7; // D8 on xiao
    let sdo = peripherals.pins.gpio9; // D10 on xiao

    let spi = SpiDeviceDriver::new_single(
        peripherals.spi2,
        sclk,
        sdo,
        None::<AnyIOPin>,
        Some(cs),
        &SpiDriverConfig::new(),
        &SpiConfig::new().baudrate(20.MHz().into()),
    )
    .unwrap();

    let dc = PinDriver::output(dc).unwrap();
    let rst = PinDriver::output(rst).unwrap();

    let mut spi_buffer: [u8; 4096] = [0; 4096];
    let di = mipidsi::interface::SpiInterface::new(spi, dc, &mut spi_buffer);

    // Display definition, invert and order colors for GC9A01
    let mut display = Builder::new(GC9A01, di)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .color_order(mipidsi::options::ColorOrder::Bgr)
        .reset_pin(rst)
        .init(&mut Ets)
        .unwrap();

    display.clear(Rgb565::RED).unwrap();

    // Setup led (if you want to change led pin you must change the type in blink_led fn)
    let mut led = PinDriver::output(peripherals.pins.gpio21).unwrap();
    let _ = led.set_high();

    // Blink led with specified frequency and repetition count
    fn blink_led(led: &mut PinDriver<'_, Gpio21, Output>, delay_ms: u32, repeat_count: u8) {
        // Blink set number of times
        for _ in 0..repeat_count {
            // Blink led on and off
            // Set low to turn on
            let _ = led.set_low();
            Delay::delay_ms(&Delay::default(), delay_ms);
            // Set high to turn off
            let _ = led.set_high();
            Delay::delay_ms(&Delay::default(), delay_ms);
        }
    }

    // https://wiki.seeedstudio.com/xiao_esp32s3_getting_started/#hardware-overview
    /* Camera
    Chip Pin	Description
    GPIO10	Camera-related clock pin
    GPIO11	Camera video data pin (Y8)
    GPIO12	Camera video data pin (Y7)
    GPIO13	Camera pixel clock pin
    GPIO14	Camera video data pin (Y6)
    GPIO15	Camera video data pin (Y2)
    GPIO16	Camera video data pin (Y5)
    GPIO17	Camera video data pin (Y3)
    GPIO18	Camera video data pin (Y4)
    GPIO40	I2C data pin for Camera
    GPIO39	I2C clock pin for Camera
    GPIO38	Camera vertical sync pin
    GPIO47	Camera horizontal sync pin
    GPIO48	Camera video data pin (Y9)
    */
    // Setup xiao esp32s3 pins for camera
    let camera = esp_camera_rs::Camera::new(
        peripherals.pins.gpio6,  // Unused
        peripherals.pins.gpio5,  // Unused
        peripherals.pins.gpio10, // gpio10 "Camera related clock pin"
        peripherals.pins.gpio15, // gpio15 pin_d0 Y2
        peripherals.pins.gpio17, // gpio17 pin_d1 Y3
        peripherals.pins.gpio18, // gpio18 pin_d2 Y4
        peripherals.pins.gpio16, // gpio16 pin_d3 Y5
        peripherals.pins.gpio14, // gpio14 pin_d4 Y6
        peripherals.pins.gpio12, // gpio12 pin_d5 Y7
        peripherals.pins.gpio11, // gpio11 pin_d6 Y8
        peripherals.pins.gpio48, // gpio48 pin_d7 Y9
        peripherals.pins.gpio38, // gpio38 pin_vsync
        peripherals.pins.gpio47, // gpio47 pin_href
        peripherals.pins.gpio13, // gpio13 pin_pclk
        peripherals.pins.gpio40, // gpio40 pin_sda
        peripherals.pins.gpio39, // gpio39 pin_scl
        10_000_000,              // Set serial clock to 10MHZ
        esp_idf_sys::ledc_timer_t_LEDC_TIMER_0,
        esp_idf_sys::ledc_channel_t_LEDC_CHANNEL_0,
        esp_camera_rs::PIXFORMAT_GRAYSCALE, // Greyscale for QR code
        FRAMESIZE,                          // 240x240 frame size
        12,                                 // JPEG Quality
        2,                                  // Frame buffer count
        esp_camera_rs::CAMERA_GRAB_LATEST,  // Grab mode: latest
    )
    .unwrap();

    // Get framebuffer map to vec, map returning an option
    fn get_framebuffer(camera: &esp_camera_rs::Camera) -> Option<Vec<u8>> {
        camera.get_framebuffer().map(|fb| fb.data().to_vec())
    }

    // Using frame buffer search for a qrcode
    fn detect(camera: &esp_camera_rs::Camera) -> Result<String, String> {
        // Get frame buffer from camera
        let frame_buffer;

        match get_framebuffer(camera) {
            Some(fb) => frame_buffer = fb,
            None => return Err("timeout".to_string()),
        };

        // Attempt to detect/decode QR from framebuffer
        let qrcode = rxing::helpers::detect_in_luma(
            frame_buffer,
            FRAME_WIDTH,
            FRAME_HEIGHT,
            Some(rxing::BarcodeFormat::QR_CODE),
        );

        // Handel successful detection and no qrcode found cases
        match qrcode {
            Ok(c) => Ok(c.getText().to_string()),
            Err(e) => {
                log::info!("No qrcode found: {}", e);
                Err("No QR found".to_string())
            }
        }
    }

    // If loop feature is enabled attempt to detect every interval
    loop {
        // Loop every 3s and detect
        {
            match get_framebuffer(&camera) {
                Some(fb) => {
                    let image = ImageRaw::<Rgb565>::new(&fb, FRAME_WIDTH);
                    Image::new(&image, Point::zero())
                        .draw(&mut display)
                        .unwrap();
                }
                None => {
                    log::error!("Camera failed no framebuffer received");
                }
            };

            // Blink quickly when code detected, once for scan
            match detect(&camera) {
                Ok(s) => {
                    log::debug!("QR detect success: \x1b[0;34m{}\x1b[0m", s);
                    blink_led(&mut led, QR_FOUND_LED_DELAY_MS, QR_FOUND_LED_BLINK_COUNT);
                }
                Err(e) => {
                    log::error!("QR detect failed: {}", e);
                    blink_led(&mut led, QR_SCAN_LED_DELAY_MS, QR_SCAN_LED_BLINK_COUNT);
                }
            }

            // Wait for detection again
            Delay::delay_ms(&Delay::default(), DETECT_INTERVAL_MS);
        }

        // Set looped to true
        if !cfg!(feature = "loop") {
            break;
        }
    }
}
