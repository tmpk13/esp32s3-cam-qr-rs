use esp_idf_hal::task::block_on;
use esp32_nimble::{uuid128, BLEDevice, BLEScan};

const SERVICE_UUID: &str = option_env!("SERVICE_UUID").expect("No uuid set");
const MSG_CHAR_UUID: &str = option_env!("SERVICE_UUID").expect("No char uuid set");
const DEVICE_NAME: &str = option_env!("DEVICE_NAME").expect("No device name set");

pub fn send_msg(msg: &str) {
    block_on(async{
        
    });
}