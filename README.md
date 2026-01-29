# Xiao ESP32s3 Sense QR-Code scanning in rust using the ESP-IDF

Initalized with template `cargo generate esp-rs/esp-idf-template cargo`

git submodule add https://github.com/jlocash/esp-camera-rs

https://github.com/jlocash/esp-camera-rs


Updated the versions for idf crates in ./esp-camera-rs/Cargo.toml
``` toml
[dependencies]
esp-idf-hal = "0.45.2"
esp-idf-sys = "0.36.1"
```

Getting error:
``` sh
E (719) i2c: i2c_set_pin(986): scl and sda gpio numbers are the same
E (729) camera: sccb init err
E (729) camera: Camera probe failed with error 0x102(ESP_ERR_INVALID_ARG)
E (739) i2c: i2c_driver_delete(481): i2c driver install error
```

Using ripgrep cli tool `rg "camera_config_t"`
I found: `esp32-camera/driver/include/esp_camera.h`

Need to connect `rxing-test-esp/target/xtensa-esp32s3-espidf/debug/build/esp-idf-sys-71e9ff740e433849/out/bindings.rs`
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


Error:
``` sh
E (879) cam_hal: cam_dma_config(301): frame buffer malloc failed
E (889) cam_hal: cam_config(385): cam_dma_config failed
E (899) camera: Camera config failed with error 0xffffffff
```
Not enough memory?

Changed target in `esp-camera-rs/.cargo/config.toml`:
`target = "xtensa-esp32s3-espidf"`
Unsure if it actually effects the build

Still breaking

Maybe enabling spiram: https://github.com/jlocash/esp-camera-rs/blob/main/sdkconfig.defaults

https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/kconfig-reference.html#component-config-esp-driver-camera-controller-configurations