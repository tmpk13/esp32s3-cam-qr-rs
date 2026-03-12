use esp32_nimble::{uuid128, BLEDevice, BLEScan};
use esp_idf_hal::task::thread::ThreadSpawnConfiguration;
use esp_idf_hal::{
    delay::Delay,
    task::block_on,
};
use std::sync::mpsc;

const DEVICE_NAME: &str = "esp-msg";

pub fn start_ble_task(rx: mpsc::Receiver<String>, done_tx: mpsc::Sender<Result<(), String>>) {
    ThreadSpawnConfiguration {
        name: Some(b"BLE\0"),
        stack_size: 12288,
        priority: 5,
        pin_to_core: Some(esp_idf_hal::cpu::Core::Core1),
        ..Default::default()
    }
    .set()
    .unwrap();

    std::thread::spawn(move || {
        let ble_device = BLEDevice::take();

        loop {
            match rx.recv() {
                Ok(text) => {
                    log::info!("BLE started");
                    let result = send_command(ble_device, &text);
                    // Send completion message, or error as string
                    let _ = done_tx.send(result.map_err(|e| e.to_string()));
                }
                Err(e) => {
                    log::error!("BLE connection broke {}", e);
                    break;
                }
            }
        }
    });
}

pub fn send_command(ble_device: &BLEDevice, msg: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Disable async, as not currently async
    block_on(async {
        let mut ble_scan = BLEScan::new();

        let device = ble_scan
            .active_scan(false)
            .interval(100)
            .window(99)
            .start(ble_device, 10000, |device, data| {
                let name = data.name();
                let has_uuid = data.service_uuids()
                    .any(|u| u == uuid128!("921a6069-4357-4287-a9af-fd386fc0dcad"));
                log::info!("Scan: name={:?} has_uuid={}", name, has_uuid);
                let found = name.map(|n| n == DEVICE_NAME).unwrap_or(false) || has_uuid;
                if found {
                    log::info!("Found target device: {:?}", device.addr());
                }
                found.then(|| *device)
            })
            .await?
            .ok_or("Device not found during scan")?;

        let mut client = ble_device.new_client();
        client.connect(&device.addr()).await?;
        let service = client.get_service(uuid128!("921a6069-4357-4287-a9af-fd386fc0dcad")).await?;
        let characteristic = service.get_characteristic(uuid128!("1ad4aa0c-5cb7-4be3-9916-9c63f19c03fd")).await?;

        characteristic.write_value(msg.as_bytes(), true).await?;
        if let Err(e) = client.disconnect() {
            log::warn!("Disconnect error (write succeeded): {}", e);
        }

        Ok(())
    })
}
