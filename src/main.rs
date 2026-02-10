/*

QR code detection and decoding

Using the rxing, esp-idf, and esp32-camera-rs crates
On the Xiao esp32s3 sense w/ camera

*/

// use esp32_nimble::BLEDevice;

mod ble;

use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{iso_8859_15::FONT_10X20, iso_8859_4::FONT_6X10, MonoFont, MonoTextStyle},
    pixelcolor::{raw::BigEndian, Rgb565},
    prelude::{Point, Primitive, RgbColor, *},
    primitives::{Arc, PrimitiveStyle, Rectangle, StyledDrawable},
    text::Text,
    Drawable,
};

use esp_idf_hal::{
    delay::Delay,
    delay::Ets,
    gpio::AnyIOPin,
    gpio::{Gpio21, Output, PinDriver},
    peripherals::Peripherals,
    spi::{SpiConfig, SpiDeviceDriver, SpiDriverConfig},
    task::thread::ThreadSpawnConfiguration,
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
const DETECT_INTERVAL_FRAMES: u32 = 30;

// Constants for the qrcode scanner set to equal frame dimensions
// FRAME_WIDTH and FRAME_HEIGHT must match dimensions of FRAMESIZE
// (see esp-camera-rs for FRAMESIZE constants)
const FRAME_WIDTH: u32 = 240;
const FRAME_HEIGHT: u32 = 240;
const FRAMESIZE: esp_idf_sys::camera::framesize_t = esp_camera_rs::FRAMESIZE_240X240;

use std::sync::mpsc;

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
        &SpiConfig::new().baudrate(70.MHz().into()),
    )
    .unwrap();

    let dc = PinDriver::output(dc).unwrap();
    let rst = PinDriver::output(rst).unwrap();

    let mut spi_buffer: [u8; 4096] = [0; 4096];
    let di = mipidsi::interface::SpiInterface::new(spi, dc, &mut spi_buffer);

    // Display definition, invert and order colors for GC9A01
    let mut display = Builder::new(GC9A01, di)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .color_order(mipidsi::options::ColorOrder::Rgb)
        .reset_pin(rst)
        .init(&mut Ets)
        .unwrap();

    // display.clear(Rgb565::WHITE).unwrap();

    // Setup led (if you want to change led pin you must change the type in blink_led fn)
    let mut led = PinDriver::output(peripherals.pins.gpio21).unwrap();
    let _ = led.set_high();

    // Blink led with specified frequency and repetition count
    fn blink_led<T: esp_idf_hal::gpio::Pin>(led: &mut PinDriver<'_, T, Output>, delay_ms: u32, repeat_count: u8) {
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
        10_000_000,              // Set serial clock to 20MHZ
        esp_idf_sys::ledc_timer_t_LEDC_TIMER_0,
        esp_idf_sys::ledc_channel_t_LEDC_CHANNEL_0,
        esp_camera_rs::PIXFORMAT_RGB565,   // Greyscale for QR code
        FRAMESIZE,                         // 240x240 frame size
        20,                                // JPEG Quality
        1,                                 // Frame buffer count
        esp_camera_rs::CAMERA_GRAB_LATEST, // Grab mode: latest
    )
    .unwrap();

    let mut framecount: u32 = 0;
    // Buffer for rxing QR code luma detect
    let mut grayscale = vec![0u8; (FRAME_WIDTH * FRAME_HEIGHT) as usize];
    // Buffer for rgb data from camera
    let mut rgb_fb = vec![0u8; (FRAME_WIDTH * FRAME_HEIGHT * 2) as usize];

    // Setup mpsc communication lines
    // tx and rx to send message to ble module
    let (tx, rx) = mpsc::channel::<String>();
    // done tx and rx to receive done state from ble module
    let (done_tx, done_rx) = mpsc::channel::<Result<(), String>>();

    // Start ble task from ble module
    ble::start_ble_task(rx, done_tx);

    // If loop feature is enabled attempt to detect every interval
    loop {
        // Add match for button vs scan loop
        // Add PIR detect match

        // Loop every interval and detect
        let frame_buffer = match camera.get_framebuffer() {
            Some(fb) => {
                log::debug!("Frame captured");
                fb
            }
            None => {
                log::error!("Timeout");
                esp_idf_hal::reset::restart();
            }
        };

        rgb_fb[..frame_buffer.data().len()].copy_from_slice(frame_buffer.data());

        drop(frame_buffer);

        let radius = 100;
        let center = Point::new((FRAME_WIDTH / 2) as i32, (FRAME_HEIGHT / 2) as i32);
        let progress = ((360.0 / DETECT_INTERVAL_FRAMES as f32) * framecount as f32).deg();

        // Generate arc
        let arc = Arc::new(
            Point {
                x: (FRAME_WIDTH / 2 - radius) as i32,
                y: (FRAME_HEIGHT / 2 - radius) as i32,
            },
            radius * 2,
            -90.0.deg(),
            progress,
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 10));

        // Add progress bar to the image
        arc.pixels().for_each(|Pixel(point, color)| {
            let index = ((point.y * FRAME_WIDTH as i32 + point.x) * 2) as usize;
            let color_u16 = color.into_storage();
            rgb_fb[index] = (color_u16 >> 8) as u8;
            rgb_fb[index + 1] = (color_u16 & 0xFF) as u8;
        });

        let image = ImageRaw::<Rgb565>::new(&rgb_fb, FRAME_WIDTH);

        Image::new(&image, Point::zero())
            .draw(&mut display)
            .unwrap();

        if framecount % DETECT_INTERVAL_FRAMES == 0 {
            // TODO: Change to get greyscale. Not correct but working
            for (i, v) in rgb_fb.chunks(2).enumerate() {
                grayscale[i] = v[0];
            }

            // Attempt to detect/decode QR from framebuffer
            let qrcode = rxing::helpers::detect_in_luma(
                grayscale.clone(),
                FRAME_WIDTH,
                FRAME_HEIGHT,
                Some(rxing::BarcodeFormat::QR_CODE),
            );
            // Handel successful detection and no qrcode found cases
            match qrcode {
                Ok(value) => {
                    let text: String = value.getText().to_string();
                    let valid_code: bool =
                        text.len() == 7 && text.bytes().all(|c| c.is_ascii_digit());
                    let text_color = if valid_code {
                        Rgb565::GREEN
                    } else {
                        Rgb565::RED
                    };

                    // Signal success in the console and via the onboard led
                    blink_led(&mut led, QR_FOUND_LED_DELAY_MS, QR_FOUND_LED_BLINK_COUNT);
                    log::info!("Qrcode found: --> {}", text);

                    Rectangle::new(
                        Point { x: 0, y: 0 },
                        Size {
                            width: FRAME_WIDTH,
                            height: FRAME_HEIGHT,
                        },
                    )
                    .draw_styled(&PrimitiveStyle::with_fill(Rgb565::BLACK), &mut display)
                    .unwrap();
                    Text::with_alignment(
                        text.as_str(),
                        center,
                        MonoTextStyle::new(&FONT_10X20, text_color),
                        embedded_graphics::text::Alignment::Center,
                    )
                    .draw(&mut display)
                    .unwrap();

                    // Check for format
                    if valid_code {
                        tx.send(text.clone()).unwrap();
                    }

                    Delay::delay_ms(&Delay::new_default(), 5000);
                }
                Err(err) => {
                    // blink_led(&mut led, QR_SCAN_LED_DELAY_MS, QR_SCAN_LED_BLINK_COUNT);

                    log::error!("No qrcode found: {}", err);
                }
            }
        }

        framecount += 1;
        if framecount > DETECT_INTERVAL_FRAMES {
            framecount = 1;
        }

        log::info!("Frame {}", framecount);

        // Set looped to true
        if !cfg!(feature = "loop") {
            break;
        }
    }
}
