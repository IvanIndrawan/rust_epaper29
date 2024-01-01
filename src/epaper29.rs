use embedded_graphics::framebuffer::{buffer_size, Framebuffer};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics_core::pixelcolor::raw::{LittleEndian, RawU1};
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use rtt_target::rprintln;

pub const WIDTH: usize = 128;
pub const HEIGHT: usize = 296;

pub const PIXEL_REGISTERS: usize = WIDTH * HEIGHT / 8;
pub type E29Buffer = Framebuffer<BinaryColor, RawU1, LittleEndian, WIDTH, HEIGHT, {buffer_size::<BinaryColor>(WIDTH, HEIGHT)}>;

pub struct E29<'d, SPI, DC, RST, BUSY, DELAY>
    where
        SPI: spi::Write<u8>,
        DC: OutputPin,
        RST: OutputPin,
        BUSY: InputPin,
        DELAY: DelayMs<u16>,
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

    black_display: E29Buffer,
    red_display: E29Buffer,
}

impl<'d, SPI, DC, RST, BUSY, DELAY> E29<'d, SPI, DC, RST, BUSY, DELAY>
    where
        SPI: spi::Write<u8>,
        DC: OutputPin,
        RST: OutputPin,
        BUSY: InputPin,
        DELAY: DelayMs<u16>,
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
        let mut black_display =  Framebuffer::<BinaryColor, RawU1, LittleEndian, WIDTH, HEIGHT, {buffer_size::<BinaryColor>(WIDTH, HEIGHT)}>::new();
        let mut red_display = Framebuffer::<BinaryColor, RawU1, LittleEndian, WIDTH, HEIGHT, {buffer_size::<BinaryColor>(WIDTH, HEIGHT)}>::new();
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
            black_display,
            red_display,
        };
        display
    }

    pub fn get_black_display(&mut self) -> &mut E29Buffer {
        return &mut self.black_display;
    }

    pub fn get_red_display(&mut self) -> &mut E29Buffer {
        return &mut self.red_display;
    }

    pub fn update_black_display(&mut self) {
        let data = self.black_display.data_mut();
        let mut data_write: [u8; PIXEL_REGISTERS] = [0; PIXEL_REGISTERS];
        data_write.copy_from_slice(&data[0..]);

        self.read_busy();
        self.write_command(0x10, &[]);

        rprintln!("Updating black display with data ");
        self.start_data();
        self.write_data(&data_write);
        // for i in 0..PIXEL_REGISTERS {
        //     self.write_data(&[data_write[i]]);
        // }
        self.end_data();
        self.delay.delay_ms(200);
    }
    pub fn update_red_display(&mut self) {
        let data_red = self.red_display.data_mut();
        let mut data_write: [u8; PIXEL_REGISTERS] = [0; PIXEL_REGISTERS];
        data_write.copy_from_slice(&data_red[0..]);

        self.read_busy();

        self.write_command(0x13, &[]);
        rprintln!("Updating red display with data ");
        self.start_data();
        self.write_data(&data_write);
        // for i in 0..PIXEL_REGISTERS {
        //     self.write_data(&[data_write[i]]);
        // }
        self.end_data();

        self.delay.delay_ms(200);
        rprintln!("Refresh display");

    }

    pub fn refresh_display(&mut self) {
        self.read_busy();
        self.write_command(0x12, &[]);
        self.read_busy();
        rprintln!("Finished rendering process 123");
    }
    pub fn init(&mut self) -> Result<(), ()>
    {
        self.hard_reset().unwrap();

        self.write_command(0x06, &[0x17, 0x17, 0x17]).unwrap();
        // self.write_data(&[0x17]).unwrap();
        // self.write_data(&[0x17]).unwrap();
        // self.write_data(&[0x17]).unwrap();

        self.write_command(0x04, &[]).unwrap();
        self.read_busy(); //waiting for the electronic paper IC to release the idle signal

        self.write_command(0x00, &[0x8f]).unwrap();   //panel setting
        // self.write_data(&[0x8f]).unwrap();

        self.write_command(0x50, &[0x77]);    //VCOM AND DATA INTERVAL SETTING
        // self.write_data(&[0x77]);   //Bmode:VBDF 17|D7 VBDW 97 VBDB 57

        self.write_command(0x61, &[0x80, 0x01, 0x28]).unwrap();    //set resolution
        // self.write_data(&[0x80]).unwrap();
        // self.write_data(&[0x01]).unwrap();
        // self.write_data(&[0x28]).unwrap();

        Ok(())
    }

    pub fn hard_reset(&mut self) -> Result<(), ()>
        where
            DELAY: DelayMs<u16>,
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
                break 'busy_loop;
            }
        }

    }
    fn write_command(&mut self, command: u8, params: &[u8]) -> Result<(), ()> {
        self.dc.set_low().map_err(|_| ())?;
        self.spi.write(&[command]);
        if !params.is_empty() {
            // self.start_data()?;
            self.write_data(params)?;
        }
        Ok(())
    }

    fn write_data(&mut self, data: &[u8]) -> Result<(), ()> {
        self.spi.write(&data);
        Ok(())
    }

    fn write_negate_data(&mut self, data: &[u8]) -> Result<(), ()> {
        let negate_data = [!data[0]];
        self.spi.write(&negate_data);
        Ok(())
    }

    fn start_data(&mut self) -> Result<(), ()> {
        self.dc.set_high();
        self.delay.delay_ms(10);
        Ok(())
    }

    fn end_data(&mut self) -> Result<(), ()> {
        self.delay.delay_ms(10);
        self.dc.set_low();
        Ok(())
    }

    pub fn clear_screen(&mut self) -> Result<(), ()> {
        self.read_busy();

        self.write_command(0x10, &[]);
        self.start_data();
        let clear_arr: [u8;PIXEL_REGISTERS] = [0x00;PIXEL_REGISTERS];
        self.spi.write(&clear_arr);

        self.end_data();

        self.write_command(0x13, &[]);
        self.start_data();
        self.spi.write(&clear_arr);
        self.end_data();

        self.write_command(0x12, &[]);

        self.delay.delay_ms(200);
        self.read_busy();
        self.delay.delay_ms(200);
        Ok(())
    }

    pub fn sleep(&mut self) -> Result<(),()> {
        rprintln!("Sleeping");
        self.write_command(0x02, &[]).unwrap(); //power off
        self.read_busy();
        self.write_command(0x07, &[0xA5]).unwrap(); // deep sleep
        // self.write_data(&[0xA5]).unwrap();

        self.delay.delay_ms(2000);
        Ok(())
    }
}