# QR-Code scanning in Rust on the Xiao *ESP32s3 Sense*
<br>


Using the Xiao Esp32s3 Sense to scan QR codes with the [**rxing**](https://github.com/rxing-core/rxing) barcode crate.  
This uses the [**esp-camera-rs**](https://github.com/jlocash/esp-camera-rs) crate modified for the Xiao esp32s3.  


*Will update with modified code from esp-camera-rs crate*

---

<br>

Initalized with [**esp-rs** template](https://github.com/esp-rs/esp-idf-template) `cargo generate esp-rs/esp-idf-template cargo`

<<<<<<< HEAD
=======
QR code crate [**rxing**](https://github.com/rxing-core/rxing)

Camera wrapper [**esp-camera-rs**](https://github.com/jlocash/esp-camera-rs)  
Modified wrapper: https://github.com/tmpk13/esp-camera-rs
>>>>>>> staging

### Features
`loop`: Attempt detection once every 3 seconds.

<img width="391" height="253" alt="image" src="https://github.com/user-attachments/assets/22eb83cf-2e0b-4efc-8aba-e066f9c0d849" />



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


<<<<<<< HEAD



# Implementation log
<details>
<summary><b>Click to expand</b></summary>

## Add wrapper
`git submodule add https://github.com/jlocash/esp-camera-rs`

## Setup esp-camera
[esp-idf remote components](https://docs.esp-rs.org/esp-idf-hal/esp_idf_sys/index.html#remote-components-idf-component-registry)
`Cargo.toml`
``` toml
[[package.metadata.esp-idf-sys.extra_components]]
remote_component = { name = "espressif/esp32-camera", version = "2.0.7" }
bindings_header = "bindings.h"
bindings_module = "camera"
```
`bindings.h`
``` c
#if defined(ESP_IDF_COMP_ESPRESSIF__ESP32_CAMERA_ENABLED)
#include "esp_camera.h"
#endif
```

## Updated esp-idf in esp-camera-rs
Updated the versions for idf crates in `./esp-camera-rs/Cargo.toml`
``` toml
[dependencies]
esp-idf-hal = "0.45.2"
esp-idf-sys = "0.36.1"
```
## Setting up I2C

*Error:*
``` sh
E (719) i2c: i2c_set_pin(986): scl and sda gpio numbers are the same
E (729) camera: sccb init err
E (729) camera: Camera probe failed with error 0x102(ESP_ERR_INVALID_ARG)
E (739) i2c: i2c_driver_delete(481): i2c driver install error
```

Found implementation for camera_config_t using ripgrep `rg "camera_config_t"`: `esp32-camera/driver/include/esp_camera.h`

Needed to connect the  `rxing-test-esp/target/xtensa-esp32s3-espidf/debug/build/esp-idf-sys-71e9ff740e433849/out/bindings.rs`
``` rust
pub struct camera_config_t {
...
    pub __bindgen_anon_1: camera_config_t__bindgen_ty_1,
    pub __bindgen_anon_2: camera_config_t__bindgen_ty_2,
...
    }
...
    pub union camera_config_t__bindgen_ty_1 {
        #[doc = "< GPIO pin for camera SDA line"]
        pub pin_sccb_sda: ::core::ffi::c_int,
        #[doc = "< GPIO pin for camera SDA line (legacy name)"]
        pub pin_sscb_sda: ::core::ffi::c_int,
    }
...
    pub union camera_config_t__bindgen_ty_2 {
        #[doc = "< GPIO pin for camera SCL line"]
        pub pin_sccb_scl: ::core::ffi::c_int,
        #[doc = "< GPIO pin for camera SCL line (legacy name)"]
        pub pin_sscb_scl: ::core::ffi::c_int,
    }
```

Added to `esp-camera-rs/src/lib.rs`:
``` rust
let config = camera::camera_config_t {
    pin_pwdn: pin_pwdn.pin(),
    pin_reset: pin_reset.pin(),
    pin_xclk: pin_xclk.pin(),

    __bindgen_anon_1: camera::camera_config_t__bindgen_ty_1 { pin_sccb_sda: pin_sccb_sda.pin() },
    __bindgen_anon_2: camera::camera_config_t__bindgen_ty_2 { pin_sccb_scl: pin_sccb_scl.pin() },
```


**Error:**
``` sh
E (879) cam_hal: cam_dma_config(301): frame buffer malloc failed
E (889) cam_hal: cam_config(385): cam_dma_config failed
E (899) camera: Camera config failed with error 0xffffffff
```
*Not enough memory?*

~~Changed target in `esp-camera-rs/.cargo/config.toml`:  
`target = "xtensa-esp32s3-espidf"`  
Unsure if it actually effects the build~~  
Still breaking doens't seems to have an effect


Maybe enabling spiram: [ESP32 config docs](https://github.com/jlocash/esp-camera-rs/blob/main/sdkconfig.defaults)

[ESP32 kconfig docs](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/kconfig-reference.html#component-config-esp-driver-camera-controller-configurations)
[Related gh issue](https://github.com/esp-rs/esp-idf-sys/issues/177)

**Needed to swap to s3**
`CONFIG_ESP32_SPIRAM_SUPPORT=y` -> `CONFIG_ESP32S3_SPIRAM_SUPPORT=y`


`sdkconfig.defaults`
``` toml
CONFIG_ESP32S3_SPIRAM_SUPPORT=y
CONFIG_SPIRAM_MODE_OCT=y
CONFIG_SPIRAM_SPEED_80M=y
CONFIG_SPIRAM_BOOT_INIT=y
CONFIG_SPIRAM_USE_MALLOC=y
CONFIG_SPIRAM_ALLOCATOR_CONTIGUITY_8K=y
```

## Frame buffer

**Error:**
``` sh
E (1440) cam_hal: FB-SIZE: 40320 != 57600
cam_hal: EV-VSYNC-OVF
...
cam_hal: EV-VSYNC-OVF
W (5463) cam_hal: Failed to get the frame on time!
```

~~**Solution:**
Set image size and shape in `esp-camera-rs/src/lib.rs`:~~
``` rust
let config = camera::camera_config_t {
...
    xclk_freq_hz: 10_000_000, // <-- for the frame buffer
...
    pixel_format: camera::pixformat_t_PIXFORMAT_GRAYSCALE,
    frame_size: camera::framesize_t_FRAMESIZE_240X240,
...
```

### Reduce clock speed

Added to `esp-camera-rs/src/lib.rs`:
``` rust
impl<'a> Camera<'a> {
    pub fn new(
...
        pin_pclk: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        
        pin_sccb_sda: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_sccb_scl: impl Peripheral<P = impl InputPin + OutputPin> + 'a,

        xclk_freq_hz: i32,

    ) -> Result<Self, esp_idf_sys::EspError> {
...
        let config = camera::camera_config_t {
...
            pin_vsync: pin_vsync.pin(),
            pin_href: pin_href.pin(),
            pin_pclk: pin_pclk.pin(),
            

            xclk_freq_hz: xclk_freq_hz,
            ledc_timer: esp_idf_sys::ledc_timer_t_LEDC_TIMER_0,
...
        };
```

Needed to reduce from `20MHZ` to `10MHZ` to fit the framebuffer. 
*Too fast?*


The frames were set by going into the library
But would be nice to set from outside

Needed to determine what to put in set_xclk as arguments for `timer` `xclk`

``` rust
pub fn set_xclk(&self, timer: i32, xclk: i32) -> Result<(), EspError> {
    esp!(unsafe { (*self.sensor).set_xclk.unwrap()(self.sensor, timer, xclk) })
}
```

`target/xtensa-esp32s3-espidf/debug/build/esp-idf-sys-71e9ff740e433849/out/bindings.rs`
``` rust
...
pub set_xclk: ::core::option::Option<
    unsafe extern "C" fn(
        sensor: *mut sensor_t,
        timer: ::core::ffi::c_int,
        xclk: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int,
>,
...
```



## Crashing while scanning

```
I (1473) cam_hal: Allocating 57600 Byte frame buffer in PSRAM
I (1473) cam_hal: cam config ok
I (1483) ov2640: Set PLL: clk_2x: 1, clk_div: 3, pclk_auto: 1, pclk_div: 8
I (1563) ov2640: Set PLL: clk_2x: 1, clk_div: 3, pclk_auto: 1, pclk_div: 8

thread 'main' panicked at src/main.rs:80:107:
decodes: NotFoundException("")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

abort() was called at PC 0x420b04c6 on core 0
0x420b04c6 - std::sys::pal::unix::abort_internal
```

**Error:**
Was crashing but it was just the `Some` was not being handeled. Was using `expect` -> now using match.
From the docs:
``` docs
Returns the contained [Ok] value, consuming the self value.

Because this function may panic, its use is generally discouraged. Instead, prefer to use pattern matching and handle the [Err] case explicitly, or call unwrap_or, unwrap_or_else, or unwrap_or_default.
```
</details>
=======
>>>>>>> staging

<br>

---

#### Claude Sonnet 4.5 was used for some debuging of errors <br> No code or output was used directely or copied from LLM output
