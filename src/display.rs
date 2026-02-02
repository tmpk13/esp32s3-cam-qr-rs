use esp_idf_hal::{
    delay::{Delay, Ets},
    gpio::*,
    prelude::*,
    spi::*,
    units::FromValueType,
};

use embedded_graphics::{
    image::ImageRaw, 
    pixelcolor::Rgb565, 
    prelude::*,
};
use mipidsi::
{
    models::GC9A01, 
    Builder,
};

pub mod lcd_display {
    use esp_idf_hal::{gpio::PinDriver, peripherals, spi::SpiDeviceDriver};

    fn display(peripherals: &peripherals::Peripherals, buffer: vec<u8>) {
        let cs = peripherals.pins.gpio2; // D1 on xiao
        let rst = peripherals.pins.gpio3; // D2 on xiao
        let dc = peripherals.pins.gpio4; // D3 on xiao
        let sclk = peripherals.pins.gpio7; // D8 on xiao
        let sdo = peripherals.pins.gpio9; // D10 on xiao

        let spi = SpiDeviceDriver::new_single(
            peripherals.spi2, 
            sclk, 
            sdo, 
            sdi, 
            Some(cs), 
            bus_config, 
            config
        ).unwrap();

        let dc = PinDriver::output(dc).unwrap();
        let rst = PinDriver::output(rst).unwrap();

        let mut spi_buf: [u8; 4096] = [0; 4096];
        let di = mipidsi::inteface::SpiInterface::new(spi, dc, &mut spi_buf);

        let mut display = Builder::new(GC9A01, di)
            .reset_pin(rst)
            .init(&mut Ets)
            .unwrap();

        

        
    }
    
    
}
