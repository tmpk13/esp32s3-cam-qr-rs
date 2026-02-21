# QR-Code scanning in Rust on the Xiao *ESP32s3 Sense*
<br>


Using the Xiao Esp32s3 Sense to scan QR codes with the [**rxing**](https://github.com/rxing-core/rxing) barcode crate.  

This uses the [**esp-camera-rs**](https://github.com/jlocash/esp-camera-rs) wrapper, modified for the Xiao esp32s3.  
Modifed wrapper: https://github.com/tmpk13/esp-camera-rs/tree/Config-Access

---

In the current system this module acts as the main controller board communicating with the lock boards over BLE.
The authentication methods include Keypad, BLE, and QR code.

The design uses a PIR sensor for low power scanning. Dectecting when a user is infront of the device ready to authenticate.

![Gif of scanning a qr code](https://github.com/tmpk13/esp32s3-cam-qr-rs/blob/ble-lock/images/PXL_20260221_222057871~3.gif?raw=true)


## Custom printed PCB
Printed circuit board with JLCPCB  
<img width="200" height="400" alt="image" src="https://github.com/tmpk13/esp32s3-cam-qr-rs/blob/ble-lock/images/PXL_20260221_220432174.RAW-01.COVER~2.jpg?raw=true" />
<img width="200" height="400" alt="image" src="https://github.com/user-attachments/assets/a72c7769-4f03-4b61-afc9-129b24a1931a" />


## 3D printed case (In Progress)
Prototype pannel for lock  
<img width="800" height="611" alt="image" src="https://github.com/user-attachments/assets/912084a3-d3f5-407d-9352-82fffa5ea538" />
<img width="200" height="400" alt="image" src="https://github.com/user-attachments/assets/19ea6202-7b73-4124-9d27-1f7d7f22c3f9" />


---


<br>

Initalized with [**esp-rs** template](https://github.com/esp-rs/esp-idf-template) `cargo generate esp-rs/esp-idf-template cargo`


## Features
`loop`: Attempt detection once every *3 seconds*.

<img width="391" height="253" alt="image" src="https://github.com/user-attachments/assets/22eb83cf-2e0b-4efc-8aba-e066f9c0d849" />



## Camera config

This was tested with an `OV2640` camera sensor.  
The newer Xiao esp32s3 senses are coming with the OV3660.

**You may need to change:  
`CONFIG_OV2640_SUPPORT=y`  to `CONFIG_OV3660_SUPPORT=y`  
In `sdkconfig.defaults`**

## Testing with a QR code

The camera will flash **once** for an unsuccessful scan.  
Flashing many times **quickly** for a successful scan.

I tested with the esp32s3 in a 3D printed housing to keep the camera pointed straight.  
The scanning was **successful** at around `12 inches` for a `1 inch` QR code. 

QR code used:

*It works!*
```
██████████████        ██    ██████████████
██          ██      ████    ██          ██
██  ██████  ██  ██    ██    ██  ██████  ██
██  ██████  ██  ██  ██      ██  ██████  ██
██  ██████  ██  ██      ██  ██  ██████  ██
██          ██  ████  ██    ██          ██
██████████████  ██  ██  ██  ██████████████
                ██  ██
██  ██████████    ██  ██    ██████████
██████  ████      ████████      ██████████
        ██████  ██████  ██████      ████
████  ██  ██  ████    ████      ██  ████
████  ██  ████  ████    ██  ██████      ██
                ██      ██    ████  ██
██████████████      ████  ██        ████
██          ██  ██████        ██████████
██  ██████  ██  ██  ████      ██  ██    ██
██  ██████  ██  ██    ████████  ██████
██  ██████  ██  ██      ██  ██    ██
██          ██    ██  ████████  ██████
██████████████  ████    ██      ██    ██
```
*Generated with the `qrcode` rust crate*



---
<br>

**Todo**
- [ ] Add range detection (lidar?) for qr to make sure the code is in frame



<br>

---

Formatting with `cargo +esp fmt`

---

#### Claude models were used for some debuging of errors <br> No code or output was used directely or copied from LLM output
