use cortex_m::asm::delay;
use cortex_m::delay::Delay;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::{InputPin, OutputPin};

pub struct E29<SPI, DC, RST, BUSY>
    where
        SPI: spi::Write<u8>,
        DC: OutputPin,
        RST: OutputPin,
        BUSY: InputPin
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
}

impl<SPI, DC, RST, BUSY> E29<SPI, DC, RST, BUSY>
    where
        SPI: spi::Write<u8>,
        DC: OutputPin,
        RST: OutputPin,
        BUSY: InputPin,
{
    /// Creates a new driver instance that uses hardware SPI.
    pub fn new(
        spi: SPI,
        dc: DC,
        rst: RST,
        busy: BUSY,
        width: u32,
        height: u32,
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
        };

        display
    }

    pub fn init<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), ()>
        where
            DELAY: DelayMs<u8>,
    {
        self.hard_reset(delay).unwrap();

        self.write_command(0x04, &[]).unwrap();
        self.read_busy(delay); //waiting for the electronic paper IC to release the idle signal

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

    pub fn hard_reset<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), ()>
        where
            DELAY: DelayMs<u8>,
    {
        self.rst.set_high();
        delay.delay_ms(200);
        self.rst.set_low();
        delay.delay_ms(2);
        self.rst.set_high();
        delay.delay_ms(200);
        Ok(())
    }

    fn read_busy<DELAY>(&mut self, delay: &mut DELAY)
        where
            DELAY: DelayMs<u8>,
    {
        'busy_loop: loop {
            self.write_command(0x71, &[]);
            delay.delay_ms(200);
            if self.busy.is_high() {
                break 'busy_loop;
            }
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


    pub fn clear<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), ()> where
        DELAY: DelayMs<u8>, {
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
        delay.delay_ms(200);
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
