/*

QR code detection and decoding

Using the rxing, esp-idf, and esp32-camera-rs crates
On the Xiao esp32s3 sense w/ camera

*/

// use esp32_nimble::BLEDevice;

mod ble;

use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{iso_8859_15::FONT_10X20, MonoTextStyle},
    pixelcolor::{Rgb565},
    prelude::{Point, Primitive, RgbColor, *},
    primitives::{Arc, PrimitiveStyle, Rectangle, StyledDrawable},
    text::Text,
    Drawable,

};

use esp_idf_hal::{
    delay::Delay,
    delay::Ets,
    gpio::AnyIOPin,
    gpio::{Output, PinDriver},
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    spi::{SpiConfig, SpiDeviceDriver, SpiDriverConfig},
    units::FromValueType,
    delay::FreeRtos,
};
use mipidsi::{models::GC9A01, Builder};


// Blink count and frequency for signaling a successful scan
const QR_FOUND_LED_BLINK_COUNT: u8 = 10;
const QR_FOUND_LED_DELAY_MS: u32 = 50;

// QR code scanning delay
const DETECT_INTERVAL_FRAMES: u32 = 20;

// Constants for the qrcode scanner set to equal frame dimensions
// FRAME_WIDTH and FRAME_HEIGHT must match dimensions of FRAMESIZE
// (see esp-camera-rs for FRAMESIZE constants)
const FRAME_WIDTH: u32 = 240;
const FRAME_HEIGHT: u32 = 240;
const FRAMESIZE: esp_idf_sys::camera::framesize_t = esp_camera_rs::FRAMESIZE_240X240;

// MCP23017 I2C address (A0=A1=A2=GND → default 0x20)
const MCP_ADDR: u8 = 0x20;

// Number of digits the keypad code must be (matches QR code validation)
const KEYPAD_CODE_LEN: usize = 7;

use std::sync::mpsc;


fn loading_bar<T: DrawTarget<Color = Rgb565>>(display: &mut T, message: &str, count: u32, max: u32, center: Point)
where <T as embedded_graphics::draw_target::DrawTarget>::Error: std::fmt::Debug
{
    let progress = (count) * 240 / max;
    // display.clear(Rgb565::WHITE).unwrap();
    Rectangle::new(
        Point { x: ((FRAME_WIDTH - max)  /2 ) as i32, y: ((FRAME_HEIGHT-10)/2) as i32 },
        Size {
            width: if progress < max { progress } else {max},
            height: 20,
        },
    )
    .draw_styled(
        &PrimitiveStyle::with_fill(Rgb565::GREEN),
        display,
    )
    .unwrap();

    Text::with_alignment(
        format!("Sending: {}", message).as_str(),
        center-Point::new(0, 20),
        MonoTextStyle::new(&FONT_10X20, Rgb565::BLACK),
        embedded_graphics::text::Alignment::Center,
    )
    .draw(display)
    .unwrap();
}

/// Read the 9 keypad buttons from MCP23017 GPIOA/GPIOB.
/// Returns a 9-bit value: bits 0-7 = GA0-GA7, bit 8 = GB0.
/// Active low: 0 = button pressed, 1 = released.
fn read_mcp_buttons(i2c: &mut I2cDriver<'_>) -> u16 {
    let mut buf = [0u8; 2];
    if i2c.write_read(MCP_ADDR, &[0x12], &mut buf, 1000).is_ok() {
        let gpioa = buf[0] as u16;
        let gpiob = (buf[1] & 0x01) as u16;
        (gpiob << 8) | gpioa
    } else {
        0x01FF // All released on I2C error
    }
}

/// Map a GPIO bit index (0–8) to the keypad digit character it represents.
/// GA0–GA7 → '1'–'8', GB0 → '9'
fn bit_to_digit(bit: u8) -> char {
    match bit {
        0 => '1', 1 => '2', 2 => '3', 3 => '4',
        4 => '5', 5 => '6', 6 => '7', 7 => '8',
        8 => '9',
        _ => '?',
    }
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
        .color_order(mipidsi::options::ColorOrder::Bgr)
        .orientation(mipidsi::options::Orientation { rotation: mipidsi::options::Rotation::Deg270, mirrored: false })
        .reset_pin(rst)
        .init(&mut Ets)
        .unwrap();

    // display.clear(Rgb565::WHITE).unwrap();

    macro_rules! display_text {
        ($text:expr, $color:expr) => {
            Text::with_alignment(
                $text,
                Point::new((FRAME_WIDTH / 2) as i32, (FRAME_HEIGHT / 2) as i32),
                MonoTextStyle::new(&FONT_10X20, $color),
                embedded_graphics::text::Alignment::Center,
            )
            .draw(&mut display)
            .unwrap();
        };
    }

    // Setup led (if you want to change led pin you must change the type in blink_led fn)
    let mut led = PinDriver::output(peripherals.pins.gpio21).unwrap();
    let _ = led.set_high();

    // Blink led with specified frequency and repetition count
    fn blink_led<T: esp_idf_hal::gpio::Pin>(
        led: &mut PinDriver<'_, T, Output>,
        delay_ms: u32,
        repeat_count: u8,
    ) {
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
    // Note: gpio6 and gpio5 are passed as "Unused" by the camera library.
    // They are reclaimed below via unsafe AnyIOPin for MCP23017 I2C.
    let camera = esp_camera_rs::Camera::new(
        peripherals.pins.gpio6,  // Unused (reclaimed for MCP23017 SCK via unsafe I2C below)
        peripherals.pins.gpio5,  // Unused (reclaimed for MCP23017 SDA via unsafe I2C below)
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
        esp_camera_rs::PIXFORMAT_RGB565,   // Greyscale for QR code
        FRAMESIZE,                         // 240x240 frame size
        1,                                // JPEG Quality
        2,                                 // Frame buffer count
        esp_camera_rs::CAMERA_GRAB_LATEST, // Grab mode: latest
    )
    .unwrap();

    // MCP23017 I2C on D4/GPIO5 (SDA) and D5/GPIO6 (SCL).
    // The camera consumed gpio5 and gpio6 as "unused" (no electrical use), so we
    // recreate them here with unsafe AnyIOPin to configure the I2C peripheral.
    let i2c_sda = unsafe { AnyIOPin::new(5) };
    let i2c_scl = unsafe { AnyIOPin::new(6) };
    let mut i2c = I2cDriver::new(
        peripherals.i2c0,
        i2c_sda,
        i2c_scl,
        &I2cConfig::new().baudrate(100u32.kHz().into()),
    )
    .unwrap();

    // Configure MCP23017:
    // IODIRA = 0xFF : GA0-GA7 all inputs  (keys 1-8)
    // IODIRB = 0x01 : GB0 input           (key 9), rest don't care
    // GPPUA  = 0xFF : pull-ups on GA0-GA7 (buttons connect pin to GND)
    // GPPUB  = 0x01 : pull-up on GB0
    i2c.write(MCP_ADDR, &[0x00, 0xFF], 1000).unwrap();
    i2c.write(MCP_ADDR, &[0x01, 0x01], 1000).unwrap();
    i2c.write(MCP_ADDR, &[0x0C, 0xFF], 1000).unwrap();
    i2c.write(MCP_ADDR, &[0x0D, 0x01], 1000).unwrap();

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

    let center = Point::new((FRAME_WIDTH / 2) as i32, (FRAME_HEIGHT / 2) as i32);

    // Keypad state — tracks digits entered so far and whether we are in keypad mode
    let mut prev_gpio: u16 = read_mcp_buttons(&mut i2c); // Initialise from real state
    let mut entered_code = String::new();
    let mut in_keypad_mode = false;

    // If loop feature is enabled attempt to detect every interval
    loop {
        // --- Poll MCP23017 for new button presses ---
        let current_gpio = read_mcp_buttons(&mut i2c);
        let new_presses = prev_gpio & !current_gpio; // Bits that just went high→low (active low)
        prev_gpio = current_gpio;

        if new_presses != 0 {
            // Handle the first newly pressed button
            for bit in 0..9u8 {
                if (new_presses >> bit) & 1 == 1 {
                    in_keypad_mode = true;
                    let digit = bit_to_digit(bit);
                    entered_code.push(digit);

                    // Show the full entered code centred on a black screen
                    display.clear(Rgb565::BLACK).unwrap();
                    Text::with_alignment(
                        entered_code.as_str(),
                        center,
                        MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE),
                        embedded_graphics::text::Alignment::Center,
                    )
                    .draw(&mut display)
                    .unwrap();

                    log::info!("Key pressed: '{}', code so far: {}", digit, entered_code);

                    if entered_code.len() >= KEYPAD_CODE_LEN {
                        // Code is complete — same logic as QR code path
                        let valid_code = entered_code.bytes().all(|c| c.is_ascii_digit());
                        if valid_code {
                            tx.send("open_close_servo".to_string()).unwrap();

                            log::info!("Keypad code accepted, awaiting BLE");
                            display.clear(Rgb565::WHITE).unwrap();

                            let mut loop_count = 0u32;
                            loop {
                                match done_rx.try_recv() {
                                    Ok(Ok(())) => {
                                        display.clear(Rgb565::WHITE).unwrap();
                                        display_text!("Success", Rgb565::GREEN);
                                        Delay::delay_ms(&Delay::default(), 1000);
                                        log::info!("BLE sent OK");
                                        break;
                                    }
                                    Ok(Err(e)) => {
                                        log::error!("Ble error: {}", e);
                                        display.clear(Rgb565::WHITE).unwrap();
                                        display_text!("Device not found", Rgb565::RED);
                                        Delay::delay_ms(&Delay::default(), 2000);
                                        break;
                                    }
                                    Err(mpsc::TryRecvError::Empty) => {
                                        loading_bar(&mut display, &entered_code, loop_count, 200, center);
                                        Delay::delay_ms(&Delay::default(), 50);
                                        loop_count += 1;
                                    }
                                    Err(mpsc::TryRecvError::Disconnected) => break,
                                }
                            }
                        } else {
                            display.clear(Rgb565::BLACK).unwrap();
                            display_text!("Invalid code", Rgb565::RED);
                            Delay::delay_ms(&Delay::default(), 2000);
                        }
                        entered_code.clear();
                        in_keypad_mode = false;
                    }
                    break; // Only process the first pressed bit per poll
                }
            }
        }

        // While in keypad mode skip camera capture and QR scanning
        if in_keypad_mode {
            FreeRtos::delay_ms(50);
            if !cfg!(feature = "loop") {
                break;
            }
            continue;
        }

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

        // let radius = 115;
        // let progress = ((360.0 / DETECT_INTERVAL_FRAMES as f32) * framecount as f32).deg();

        // // Generate arc
        // let arc = Arc::new(
        //     Point {
        //         x: (FRAME_WIDTH / 2 - radius) as i32,
        //         y: (FRAME_HEIGHT / 2 - radius) as i32,
        //     },
        //     radius * 2,
        //     -90.0.deg(),
        //     progress,
        // )
        // .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 10));

        // // Add progress bar to the image
        // arc.pixels().for_each(|Pixel(point, color)| {
        //     let index = ((point.y * FRAME_WIDTH as i32 + point.x) * 2) as usize;
        //     let color_u16 = color.into_storage();
        //     rgb_fb[index] = (color_u16 >> 8) as u8;
        //     rgb_fb[index + 1] = (color_u16 & 0xFF) as u8;
        // });

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

                    display.clear(Rgb565::BLACK).unwrap();
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
                        // tx.send(text.clone()).unwrap();
                        tx.send("open_close_servo".to_string()).unwrap();

                        log::info!("For BLE connection");
                        display.clear(Rgb565::WHITE).unwrap();


                        let mut loop_count = 0;
                        loop {
                            match done_rx.try_recv() {
                                Ok(Ok(())) => {
                                    // Success message sent
                                    display.clear(Rgb565::WHITE).unwrap();
                                    display_text!("Success", Rgb565::GREEN);
                                    Delay::delay_ms(&Delay::default(), 1000);
                                    log::info!("BLE sent OK");
                                    break;
                                }
                                Ok(Err(e)) => {
                                    // Recieved error from ble
                                    log::error!("Ble error: {}", e);

                                    display.clear(Rgb565::WHITE).unwrap();
                                    display_text!("Device not found", Rgb565::RED);
                                    Delay::delay_ms(&Delay::default(), 2000);

                                    break;
                                }
                                Err(mpsc::TryRecvError::Empty) => {
                                    // No response loop
                                    log::info!("Waiting... {}/200", loop_count);

                                    loading_bar(&mut display, &text, loop_count, 200, center);

                                    Delay::delay_ms(&Delay::default(), 50);
                                    loop_count += 1;
                                }
                                Err(mpsc::TryRecvError::Disconnected) => {
                                    // Ble disconnected
                                    break;
                                }
                            }
                        }
                    }

                    Delay::delay_ms(&Delay::new_default(), 3000);
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
        FreeRtos::delay_ms(1);
    }
}
