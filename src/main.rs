use esp_idf_sys;

use esp_camera_rs;


fn main() {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71

    use rxing::qrcode::encoder::qrcode_encoder;

    use rxing::qrcode::decoder::{qrcode_decoder, ErrorCorrectionLevel};

    let matrix = qrcode_encoder::encode("HI", ErrorCorrectionLevel::L).expect("Encode failed");

    let string = qrcode_decoder::decode_bitmatrix(&matrix
            .getMatrix()
            .as_ref()
            .unwrap()
            .clone()
            .try_into()
            .expect("convert"),).expect("decode");
    
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
    let camera = esp_camera_rs::Camera::new(pin_pwdn, pin_reset, pin_xclk, pin_d0, pin_d1, pin_d2, pin_d3, pin_d4, pin_d5, pin_d6, pin_d7, pin_vsync, pin_href, pin_pclk);

    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Code: {:?}", string.getText());
}
