use std::str::from_utf8;

use esp_idf_hal::task::block_on;
use esp32_nimble::{uuid128, BLEDevice, BLEScan};

const SERVICE_UUID: &str = option_env!("SERVICE_UUID").expect("No uuid set");
const MSG_CHAR_UUID: &str = option_env!("SERVICE_UUID").expect("No char uuid set");
const DEVICE_NAME: &str = option_env!("DEVICE_NAME").expect("No device name set");

pub fn send_msg(msg: &str) -> Result<(), &str> {
    block_on(async{
        let ble_device = BLEDevice::take();
        let mut ble_scan = BLEScan::new();

        let device = ble_scan
            .active_scan(true)
            .interval(100)
            .window(99)
            .start(ble_device, 10000, |device, data| {
                data.name()
                    .and_then(|name| from_utf8(name).ok())
                    .filter(|&name| name == DEVICE_NAME)
                    .map(|_| *device)
            })
            .await?
            .ok_or("Device not found")?;
        
        let mut client = ble_device.new_client();
        client.connect(&device.addr()).await.unwrap();
        let service = client.get_service(uuid128!(SERVICE_UUID)).await.unwrap();
        let characteristic = service.get_characteristic(uuid128!(MSG_CHAR_UUID)).await.unwrap();

        characteristic.write_value(msg.as_bytes(), true).await.unwrap();

        Ok(())
    })
}