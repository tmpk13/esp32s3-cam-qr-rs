use esp32_nimble::{uuid128, BLEDevice, BLEScan};
use esp_idf_hal::{delay, task::block_on};

const SERVICE_UUID: &str = "921a6069-4357-4287-a9af-fd386fc0dcad";
const MSG_CHAR_UUID: &str = "1ad4aa0c-5cb7-4be3-9916-9c63f19c03fd";
const DEVICE_NAME: &str = "esp-msg";

pub fn send_command(
    msg: &str,
    delay: &esp_idf_hal::delay::Delay,
) -> Result<(), Box<dyn std::error::Error>> {
    // Disable async, as not currently async
    block_on(async {
        let ble_device = BLEDevice::take();
        let mut ble_scan = BLEScan::new();

        let device = ble_scan
            .active_scan(true)
            .interval(100)
            .window(99)
            .start(ble_device, 10000, |device, data| {
                data.name()
                    .filter(|&name| name == DEVICE_NAME)
                    .map(|_| *device)
            })
            .await?
            .ok_or("Device not found")?;

        let mut client = ble_device.new_client();
        client.connect(&device.addr()).await?;
        let service = client.get_service(uuid128!(SERVICE_UUID)).await?;
        let characteristic = service.get_characteristic(uuid128!(MSG_CHAR_UUID)).await?;

        characteristic.write_value(msg.as_bytes(), true).await?;

        // Wait to avoid exiting before message is sent
        delay::Delay::delay_ms(&delay, 5000);
        client.disconnect()?;

        Ok(())
    })
}
