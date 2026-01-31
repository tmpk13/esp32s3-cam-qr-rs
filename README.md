# QR-Code scanning in Rust on the Xiao *ESP32s3 Sense*
<br>

Initalized with [**esp-rs** template](https://github.com/esp-rs/esp-idf-template) `cargo generate esp-rs/esp-idf-template cargo`

QR code crate [**rxing**](https://github.com/rxing-core/rxing)

Camera wrapper [**esp-camera-rs**](https://github.com/jlocash/esp-camera-rs)  
Modified wrapper: https://github.com/tmpk13/esp-camera-rs

## Features
`loop`: Attempt detection once every *3 seconds*.



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
<br>

---

### Claude Sonnet 4.5 was used for some debuging of errors <br> No code or output was used directely or copied from LLM output
