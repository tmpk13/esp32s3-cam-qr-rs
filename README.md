# QR-Code scanning in Rust on the Xiao *ESP32s3 Sense*
<br>

Initalized with [**esp-rs** template](https://github.com/esp-rs/esp-idf-template) `cargo generate esp-rs/esp-idf-template cargo`

QR code crate [**rxing**](https://github.com/rxing-core/rxing)

Camera wrapper [**esp-camera-rs**](https://github.com/jlocash/esp-camera-rs)  
Modified wrapper: https://github.com/tmpk13/esp-camera-rs

### Features
`loop`: Attempt detection once every 3 seconds.



# Camera config
This was tested with an `OV2640` camera sensor.  
The newer Xiao esp32s3 senses are coming with the OV3660.

**You may need to change:  
`CONFIG_OV2640_SUPPORT=y`  to `CONFIG_OV3660_SUPPORT=y`  
In `sdkconfig.defaults`**

## Example QR code
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
