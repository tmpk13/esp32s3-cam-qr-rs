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

    // let camera = esp_camera_rs::Camera::new(

    // );

    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Code: {:?}", string.getText());
}
