#![no_std]
#![no_main]

use rtt_target::{rtt_init_print, rprintln};

// The trait used by formatting macros like write! and writeln!
use core::fmt::Write as FmtWrite;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::primitives::{Circle, Line, Primitive, PrimitiveStyle};
use embedded_graphics::text::{Baseline, Text, TextStyleBuilder};
use embedded_graphics_core::Drawable;
use embedded_graphics_core::geometry::Point;
use embedded_graphics_core::pixelcolor::BinaryColor;

use embedded_hal::digital::v2::OutputPin;
// The macro for our start-up function
use rp_pico::entry;

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Pull in any important traits
use rp_pico::hal::prelude::*;

// A shorter alias for the Peripheral Access Crate, which provides low-level
// register access
use rp_pico::hal::{gpio, pac, spi};

// A shorter alias for the Hardware Abstraction Layer, which provides
// higher-level drivers.
use rp_pico::hal;
use rp_pico::hal::fugit::RateExtU32;
use rust_epaper29::epaper29::E29;

/// Entry point to our bare-metal application.
///
/// The `#[entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables are initialised.
///
/// The function configures the RP2040 peripherals, then blinks the LED in an
/// infinite loop.
#[entry]
unsafe fn main() -> ! {


    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
        .ok()
        .unwrap();

    // The delay object lets us wait for specified amounts of time (in
    // milliseconds)
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    rtt_init_print!();

    // Set up our SPI pins as per  https://www.waveshare.com/2.9inch-e-paper-module-b.htm
    let epaper_dc = pins.gpio8.into_push_pull_output();
    let epaper_clock = pins.gpio10.into_function::<hal::gpio::FunctionSpi>();
    let epaper_mosi = pins.gpio11.into_function::<hal::gpio::FunctionSpi>();
    let epaper_reset = pins.gpio12.into_push_pull_output_in_state(hal::gpio::PinState::High);
    let epaper_busy = pins.gpio13.into_pull_up_input();

    let spi_uninit = hal::Spi::<_, _, _, 8>::new(pac.SPI1, (epaper_mosi, epaper_clock));
    // Exchange the uninitialised SPI driver for an initialised one
    let spi = spi_uninit.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        10.MHz(),
        embedded_hal::spi::MODE_0,
    );

    let mut screen = E29::new(spi, epaper_dc, epaper_reset, epaper_busy, 128, 296, &mut delay);

    rprintln!("Initialising");
    screen.init();
    rprintln!("Clearing screen");
    screen.clear_screen();



    rprintln!("Start drawing");

    //write a line here
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .background_color(BinaryColor::Off)
        .text_color(BinaryColor::On)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();
    let nextPos = Text::with_text_style("Hello world!", Point::new(10, 10), style, text_style).draw(screen.get_red_display()).unwrap();

    let _ = Line::new(Point::new(64, 64), Point::new(30, 40))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 4))
        .draw(screen.get_red_display());

    let _ = Circle::with_center(Point::new(64, 64), 80)
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(screen.get_red_display());

    let _ = Circle::with_center(Point::new(64, 144), 80)
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 2))
        .draw(screen.get_black_display());

    rprintln!("Next position is {}, {}", nextPos.x, nextPos.y);
    screen.update_black_display();
    screen.update_red_display();
    // screen.update_black_display();
    screen.refresh_display();

    rprintln!("Going to sleep");
    screen.sleep();
    rprintln!("Sleeping now");
    loop {
        delay.delay_ms(100);
    }

}