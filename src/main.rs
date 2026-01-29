use esp_camera_rs;
use esp_idf_hal;
use esp_idf_sys::camera::framesize_t_FRAMESIZE_240X240;
use rxing::{BarcodeFormat};

fn main() {
    // https://wiki.seeedstudio.com/xiao_esp32s3_getting_started/#hardware-overview
    /*
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
    let peripherals = esp_idf_hal::peripherals::Peripherals::take().unwrap();
    let camera = esp_camera_rs::Camera::new(
        peripherals.pins.gpio1,  // Unused
        peripherals.pins.gpio2,  // Unused
        peripherals.pins.gpio10, // "Camera related clock pin"
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
    )
    .unwrap();

    camera
        .sensor()
        .set_framesize(framesize_t_FRAMESIZE_240X240)
        .unwrap();

    let fb = camera.get_framebuffer().unwrap();
    
    let code = rxing::helpers::detect_in_luma(fb.data().to_vec(), 240, 240, Some(BarcodeFormat::QR_CODE));

    

    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    match code {
        Ok(c) => log::info!("QRcode: {}", c.getText()),
        Err(e) => log::info!("No qrcode found {}", e),
    }
}
