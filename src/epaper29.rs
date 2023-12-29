use cortex_m::asm::delay;
use cortex_m::delay::Delay;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::Pixel;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::primitives::Rectangle;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use rtt_target::rprintln;

pub struct E29<'d, SPI, DC, RST, BUSY, DELAY>
    where
        SPI: spi::Write<u8>,
        DC: OutputPin,
        RST: OutputPin,
        BUSY: InputPin,
        DELAY: DelayMs<u8>,
{
    /// SPI
    spi: SPI,

    /// Data/command pin.
    dc: DC,

    /// Reset pin.
    rst: RST,

    /// Reset pin.
    busy: BUSY,

    /// Global image offset
    dx: u16,
    dy: u16,
    width: u32,
    height: u32,

    ///delay
    delay: &'d mut DELAY,
}

impl<'d, SPI, DC, RST, BUSY, DELAY> E29<'d, SPI, DC, RST, BUSY, DELAY>
    where
        SPI: spi::Write<u8>,
        DC: OutputPin,
        RST: OutputPin,
        BUSY: InputPin,
        DELAY: DelayMs<u8>,
{
    /// Creates a new driver instance that uses hardware SPI.
    pub fn new(
        spi: SPI,
        dc: DC,
        rst: RST,
        busy: BUSY,
        width: u32,
        height: u32,
        delay: &'d mut DELAY,
    ) -> Self {
        let display = E29 {
            spi,
            dc,
            rst,
            busy,
            dx: 0,
            dy: 0,
            width,
            height,
            delay,
        };
        display
    }

    pub fn init(&mut self) -> Result<(), ()>
    {
        self.hard_reset().unwrap();

        self.write_command(0x04, &[]).unwrap();
        self.read_busy(); //waiting for the electronic paper IC to release the idle signal

        self.write_command(0x00, &[]).unwrap();   //panel setting
        self.write_data(&[0x0f]).unwrap();   //LUT from OTP,128x296
        self.write_data(&[0x89]).unwrap();    //Temperature sensor, boost and other related timing settings

        self.write_command(0x61, &[]).unwrap();    //set resolution
        self.write_data(&[0x80]).unwrap();
        self.write_data(&[0x01]).unwrap();
        self.write_data(&[0x28]).unwrap();

        self.write_command(0x50, &[]);    //VCOM AND DATA INTERVAL SETTING
        self.write_data(&[0x77]);   //Bmode:VBDF 17|D7 VBDW 97 VBDB 57
        Ok(())
    }

    pub fn hard_reset(&mut self) -> Result<(), ()>
        where
            DELAY: DelayMs<u8>,
    {
        self.rst.set_high();
        self.delay.delay_ms(200);
        self.rst.set_low();
        self.delay.delay_ms(2);
        self.rst.set_high();
        self.delay.delay_ms(200);
        Ok(())
    }

    fn read_busy(&mut self)
    {
        let mut x = 0;
        'busy_loop: loop {
            self.write_command(0x71, &[]);
            self.delay.delay_ms(200);
            x=x+1;
            if self.busy.is_high().map_err(|_| false).unwrap() {
                rprintln!("Busy pin is released");
                break 'busy_loop;
            }
            if x>30 {
                rprintln!("waiting for too long");
                break 'busy_loop;
            }
            rprintln!("Still busy");

        }

    }
    fn write_command(&mut self, command: u8, params: &[u8]) -> Result<(), ()> {
        self.dc.set_low().map_err(|_| ())?;
        self.spi.write(&[command]);
        if !params.is_empty() {
            self.start_data()?;
            self.write_data(params)?;
        }
        Ok(())
    }

    fn write_data(&mut self, data: &[u8]) -> Result<(), ()> {
        self.spi.write(data);
        Ok(())
    }

    fn start_data(&mut self) -> Result<(), ()> {
        self.dc.set_high();
        Ok(())
    }

    fn end_data(&mut self) -> Result<(), ()> {
        self.dc.set_low();
        Ok(())
    }


    pub fn clear_screen(&mut self) -> Result<(), ()> {
        self.write_command(0x10, &[]);
        self.start_data();
        for i in 0..4736 {
            self.write_data(&[0xff]);
        }
        self.end_data();
        self.write_command(0x13, &[]);
        self.start_data();
        for i in 0..4736 {
            self.spi.write(&[0xff]);
        }
        self.end_data();
        self.write_command(0x12, &[]);
        self.delay.delay_ms(200);
        Ok(())
    }

    /// Writes a data word to the display.
    fn write_word(&mut self, value: u16) -> Result<(), ()> {
        self.write_data(&value.to_be_bytes())
    }

    fn write_words_buffered(&mut self, words: impl IntoIterator<Item=u16>) -> Result<(), ()> {
        let mut buffer = [0; 32];
        let mut index = 0;
        for word in words {
            let as_bytes = word.to_be_bytes();
            buffer[index] = as_bytes[0];
            buffer[index + 1] = as_bytes[1];
            index += 2;
            if index >= buffer.len() {
                self.write_data(&buffer)?;
                index = 0;
            }
        }
        self.write_data(&buffer[0..index])
    }
}

impl<'d, SPI, DC, RST, BUSY, DELAY> Dimensions for E29<'d, SPI, DC, RST, BUSY, DELAY>
    where
        SPI: spi::Write<u8>,
        DC: OutputPin,
        RST: OutputPin,
        BUSY: InputPin,
        DELAY: DelayMs<u8>,
{
    fn bounding_box(&self) -> Rectangle {
        todo!()
    }
}

impl <'d,SPI, DC, RST, BUSY, DELAY> DrawTarget for E29<'d, SPI, DC, RST, BUSY, DELAY>
    where
        SPI: spi::Write<u8>,
        DC: OutputPin,
        RST: OutputPin,
        BUSY: InputPin,
        DELAY: DelayMs<u8>,
{
    type Color = BinaryColor;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error> where I: IntoIterator<Item=Pixel<Self::Color>> {
        todo!()
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error> where I: IntoIterator<Item=Self::Color> {
        todo!()
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        todo!()
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.clear_screen()
    }
}
//     fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
//         where
//             I: IntoIterator<Item = Self::Color>,
//     {
//         // Clamp area to drawable part of the display target
//         let drawable_area = area.intersection(&Rectangle::new(Point::zero(), self.size()));
//
//         if drawable_area.size != Size::zero() {
//             self.set_pixels_buffered(
//                 drawable_area.top_left.x as u16,
//                 drawable_area.top_left.y as u16,
//                 (drawable_area.top_left.x + (drawable_area.size.width - 1) as i32) as u16,
//                 (drawable_area.top_left.y + (drawable_area.size.height - 1) as i32) as u16,
//                 area.points()
//                     .zip(colors)
//                     .filter(|(pos, _color)| drawable_area.contains(*pos))
//                     .map(|(_pos, color)| RawU16::from(color).into_inner()),
//             )?;
//         }
//
//         Ok(())
//     }
//
//     fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
//         self.set_pixels_buffered(
//             0,
//             0,
//             self.width as u16 - 1,
//             self.height as u16 - 1,
//             core::iter::repeat(RawU16::from(color).into_inner())
//                 .take((self.width * self.height) as usize),
//         )
//     }
// }