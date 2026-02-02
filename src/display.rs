

pub mod lcd_display {
    use esp_idf_hal::{
        delay::Ets, gpio::AnyIOPin, spi::*, units::FromValueType
    };

    use embedded_graphics::{
        image::ImageRaw, 
        pixelcolor::Rgb565, 
    };
    use mipidsi::
    {
        models::GC9A01, 
        Builder,
    };
    use embedded_graphics::image::ImageDrawable;
    use esp_idf_hal::{gpio::PinDriver, peripherals, spi::SpiDeviceDriver};

    pub fn display(spi2: AnyIOPin, sclk, sdo, cs, dc, rst, buffer: &[u8], img_width: u32) {
        
        let spi = SpiDeviceDriver::new_single(
            spi2, 
            sclk, 
            sdo, 
            None::<AnyIOPin>, 
            Some(cs), 
            &SpiDriverConfig::new(),
            &SpiConfig::new().baudrate(40.MHz().into()),
        ).unwrap();

        let dc = PinDriver::output(dc).unwrap();
        let rst = PinDriver::output(rst).unwrap();

        let mut spi_buffer: [u8; 4096] = [0; 4096];
        let di = mipidsi::interface::SpiInterface::new(spi, dc, &mut spi_buffer);

        let mut display = Builder::new(GC9A01, di)
            .reset_pin(rst)
            .init(&mut Ets)
            .unwrap();

        let _image = ImageRaw::<Rgb565>::new(buffer, img_width)
            .draw(&mut display);
        
        
    }
    
    
}
