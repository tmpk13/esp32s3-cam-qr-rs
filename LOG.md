# Implementation log

## Setup esp-camera binding
[esp-idf remote components](https://docs.esp-rs.org/esp-idf-hal/esp_idf_sys/index.html#remote-components-idf-component-registry)
Placed in `Cargo.toml`
``` toml
[[package.metadata.esp-idf-sys.extra_components]]
remote_component = { name = "espressif/esp32-camera", version = "2.0.7" }
bindings_header = "bindings.h"
bindings_module = "camera"
```
Add
`bindings.h` to project base directory
``` c
#if defined(ESP_IDF_COMP_ESPRESSIF__ESP32_CAMERA_ENABLED)
#include "esp_camera.h"
#endif
```

## Updated esp-idf in esp-camera-rs
Updated the versions for idf crates in ./esp-camera-rs/Cargo.toml
``` toml
[dependencies]
esp-idf-hal = "0.45.2"
esp-idf-sys = "0.36.1"
```
## Setting up I2C

*Found Error:*
``` sh
E (719) i2c: i2c_set_pin(986): scl and sda gpio numbers are the same
E (729) camera: sccb init err
E (729) camera: Camera probe failed with error 0x102(ESP_ERR_INVALID_ARG)
E (739) i2c: i2c_driver_delete(481): i2c driver install error
```

Found implementation for camera_config_t  
~~Using ripgrep `rg "camera_config_t"`: `esp32-camera/driver/include/esp_camera.h`~~

Needed to connect the serial bindings found in `esp32s3-cam-qr-rs/target/xtensa-esp32s3-espidf/debug/build/esp-idf-sys-71e9ff740e433849/out/bindings.rs`
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

Added access to bindings in config to `esp-camera-rs/src/lib.rs`:
``` rust
let config = camera::camera_config_t {
    pin_pwdn: pin_pwdn.pin(),
    pin_reset: pin_reset.pin(),
    pin_xclk: pin_xclk.pin(),

    __bindgen_anon_1: camera::camera_config_t__bindgen_ty_1 { pin_sccb_sda: pin_sccb_sda.pin() },
    __bindgen_anon_2: camera::camera_config_t__bindgen_ty_2 { pin_sccb_scl: pin_sccb_scl.pin() },
```


**Error Found:**
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


Enabling spiram: [ESP32 config docs](https://github.com/jlocash/esp-camera-rs/blob/main/sdkconfig.defaults)

[ESP32 kconfig docs](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/kconfig-reference.html#component-config-esp-driver-camera-controller-configurations)
[Related gh issue](https://github.com/esp-rs/esp-idf-sys/issues/177)

**Needed to swap to s3 config:**  
`CONFIG_ESP32_SPIRAM_SUPPORT=y` -> `CONFIG_ESP32S3_SPIRAM_SUPPORT=y`


In `sdkconfig.defaults` added:
``` toml
CONFIG_ESP32S3_SPIRAM_SUPPORT=y
CONFIG_SPIRAM_MODE_OCT=y
CONFIG_SPIRAM_SPEED_80M=y
CONFIG_SPIRAM_BOOT_INIT=y
CONFIG_SPIRAM_USE_MALLOC=y
CONFIG_SPIRAM_ALLOCATOR_CONTIGUITY_8K=y
```

## Frame buffer

**Error Found:**
``` sh
E (1440) cam_hal: FB-SIZE: 40320 != 57600
cam_hal: EV-VSYNC-OVF
...
cam_hal: EV-VSYNC-OVF
W (5463) cam_hal: Failed to get the frame on time!
```

**Solution:**
Set image size and shape in `esp-camera-rs/src/lib.rs`:
``` rust
let config = camera::camera_config_t {
...
    xclk_freq_hz: 10_000_000, // <-- for the frame buffer
...
    pixel_format: camera::pixformat_t_PIXFORMAT_GRAYSCALE,
    frame_size: camera::framesize_t_FRAMESIZE_240X240,
...
```



## Added frame acess
The frames were set by going into the library

Needed to reduce from `20MHZ` to `10MHZ` to fit the framebuffer.  
*Too fast?*

Need to determine what to put in set_xclk as arguments for `timer` `xclk`

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
Was crashing but it was just the `Some` was not being handeled.  
Was using `expect` -> now using match.  
From the docs:
``` docs
Returns the contained [Ok] value, consuming the self value.

Because this function may panic, its use is generally discouraged. Instead, prefer to use pattern matching and handle the [Err] case explicitly, or call unwrap_or, unwrap_or_else, or unwrap_or_default.
```

*Error Found:* no target, linker
Added `rust-toolchain.toml` to repo
Added `build.rs` to repo

# Display

## Wrong color on output
Need to invert and swap color order for Gc9a01

Convert greyscale to 565
u8 in xxxx xxxx
r = 3 >> : 000x xxxx (5 remain)
g = 3 >> : 00xx xxxx (6 remain)
b = 3 >> : 000x xxxx (5 remain)
*Losing the bottom 2-3 bits (integers 0-8)*

    << 11   << 5  << 0
    \/     \/    \/
xxxxx 000000 00000 r
00000 xxxxxx 00000 g
00000 000000 xxxxx b

| (Or) combines. Each value is shifted. All zeros for the others for a given section

Clone and append bytes. Split from 16 bits to 2x 8 bits. xxxxxxxxxxxxxxxx to xxxxxxxx xxxxxxxx
To litte endian bytes --> [u8; 2]
Extend add both at farthest unused --> Vec[..., u8, u8, ...]

## Stack overflow
When adding the display to the camera receiving error about stack overflow  
Most likely camera and lcd together using more than reserved  
Present size: `CONFIG_ESP_MAIN_TASK_STACK_SIZE=8000`  
Trying: `CONFIG_ESP_MAIN_TASK_STACK_SIZE=16000`  
[!] Should find a better solution 

Running in release would probably help...

## Frame buffer timing out
Fixed by reducing display spi speed to `20MHz`. *(Doubt this was needed)*



**What probably did it was changing the camera buffer count from `1` to `2`.**

## Image not in greyscale
Add greyscale to 565 u8 x2

## Now that using a greyscale image should stop swaping color? 


## Timing out after first capture
As frame buffer is increased the number of frames before hitting an error is also increased equally
Probably not getting rid of the old frames
*Life time issue?*
https://doc.rust-lang.org/rust-by-example/scope/lifetime/explicit.html

Remove wrapper: did not fix it
Drop framebuffer : did not work
Reuse buffer : did not work

Need to update esp-camera-rs to return buffer:
https://github.com/espressif/esp32-camera/blob/fb7b85b2b79fb039551c67d295e884d2b1eb907b/driver/esp_camera.c#L404
https://components.espressif.com/components/espressif/esp32-camera/versions/2.1.4/readme

Watch dog is triggering on large spi writes. Probably due to using Ets.
Lowering camera clock speed seems to fix the issue (15 MHz -> 10 MHz)