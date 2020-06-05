#![no_std]
#![no_main]

use core::fmt::Write;

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use cortex_m_rt::entry;

use embedded_hal as hal;
use hal::digital::v2::OutputPin;

use stm32g0xx_hal::{
    prelude::*,
    stm32,
    spi,
    serial::Config,
    gpio,
    exti::Event,
    rcc
};

use nb::block;

use ssd1362::{self, display::DisplayRotation};


use embedded_graphics::{
    fonts::{Font6x8, Text, Font24x32},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Rectangle},
    style::{PrimitiveStyle, TextStyle, TextStyleBuilder},
};


const I: [u8; 32] = [
  /*
     * code=73, hex=0x49, ascii="I"
     */
    0x1F,0x80,  /* 000111111000 */
    0x1F,0x80,  /* 000111111000 */
    0x06,0x00,  /* 000001100000 */
    0x06,0x00,  /* 000001100000 */
    0x06,0x00,  /* 000001100000 */
    0x06,0x00,  /* 000001100000 */
    0x06,0x00,  /* 000001100000 */
    0x06,0x00,  /* 000001100000 */
    0x06,0x00,  /* 000001100000 */
    0x06,0x00,  /* 000001100000 */
    0x06,0x00,  /* 000001100000 */
    0x06,0x00,  /* 000001100000 */
    0x1F,0x80,  /* 000111111000 */
    0x1F,0x80,  /* 000111111000 */
    0x00,0x00,  /* 000000000000 */
    0x00,0x00,  /* 000000000000 */
    ];
    const J: [u8; 32] = [
    /*
     * code=74, hex=0x4A, ascii="J"
     */
    0x00,0x60,  /* 000000000110 */
    0x00,0x60,  /* 000000000110 */
    0x00,0x60,  /* 000000000110 */
    0x00,0x60,  /* 000000000110 */
    0x00,0x60,  /* 000000000110 */
    0x00,0x60,  /* 000000000110 */
    0x00,0x60,  /* 000000000110 */
    0x00,0x60,  /* 000000000110 */
    0x00,0x60,  /* 000000000110 */
    0x60,0x60,  /* 011000000110 */
    0x60,0x60,  /* 011000000110 */
    0x70,0xC0,  /* 011100001100 */
    0x3F,0xC0,  /* 001111111100 */
    0x1F,0x80,  /* 000111111000 */
    0x00,0x00,  /* 000000000000 */
    0x00,0x00,  /* 000000000000 */
    ];
    const T: [u8; 32] = [
        0x3F,0xC0,  /* 001111111100 */
        0x3F,0xC0,  /* 001111111100 */
        0x06,0x00,  /* 000001100000 */
        0x06,0x00,  /* 000001100000 */
        0x06,0x00,  /* 000001100000 */
        0x06,0x00,  /* 000001100000 */
        0x06,0x00,  /* 000001100000 */
        0x06,0x00,  /* 000001100000 */
        0x06,0x00,  /* 000001100000 */
        0x06,0x00,  /* 000001100000 */
        0x06,0x00,  /* 000001100000 */
        0x06,0x00,  /* 000001100000 */
        0x06,0x00,  /* 000001100000 */
        0x06,0x00,  /* 000001100000 */
        0x00,0x00,  /* 000000000000 */
        0x00,0x00,  /* 000000000000 */
        ];
    const R: [u8; 32] = [
        0x7F,0x80,  /* 011111111000 */
        0x7F,0xC0,  /* 011111111100 */
        0x60,0xE0,  /* 011000001110 */
        0x60,0x60,  /* 011000000110 */
        0x60,0x60,  /* 011000000110 */
        0x60,0x60,  /* 011000000110 */
        0x60,0xE0,  /* 011000001110 */
        0x7F,0xC0,  /* 011111111100 */
        0x7F,0x80,  /* 011111111000 */
        0x67,0x00,  /* 011001110000 */
        0x63,0x80,  /* 011000111000 */
        0x61,0xC0,  /* 011000011100 */
        0x60,0xE0,  /* 011000001110 */
        0x60,0x60,  /* 011000000110 */
        0x00,0x00,  /* 000000000000 */
        0x00,0x00,  /* 000000000000 */
        ];
        const E: [u8; 32] = [
            0x7F,0xE0,  /* 011111111110 */
            0x7F,0xE0,  /* 011111111110 */
            0x60,0x00,  /* 011000000000 */
            0x60,0x00,  /* 011000000000 */
            0x60,0x00,  /* 011000000000 */
            0x60,0x00,  /* 011000000000 */
            0x7F,0x80,  /* 011111111000 */
            0x7F,0x80,  /* 011111111000 */
            0x60,0x00,  /* 011000000000 */
            0x60,0x00,  /* 011000000000 */
            0x60,0x00,  /* 011000000000 */
            0x60,0x00,  /* 011000000000 */
            0x7F,0xE0,  /* 011111111110 */
            0x7F,0xE0,  /* 011111111110 */
            0x00,0x00,  /* 000000000000 */
            0x00,0x00,  /* 000000000000 */
        ];


#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("cannot take peripherals");

    // let cfg = rcc::Config::pll();

    // default pll config is 64MHz

    // f_vco = 16 / 4 * 30 = 120
    // f = 120 / 2 = 60
    // let pll_cfg = rcc::PllConfig::with_hsi(4, 20, 2 );

    let pll_cfg = rcc::PllConfig::default();
    let rcc_cfg = rcc::Config::pll().pll_cfg(pll_cfg);
    let mut rcc = dp.RCC.freeze(rcc_cfg);

    // let mut rcc = dp.RCC.constrain();

    let mut delay = dp.TIM15.delay(&mut rcc);

    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);
    let gpioc = dp.GPIOC.split(&mut rcc);

    let btn = gpioc.pc13.into_pull_up_input();
    let mut exti = dp.EXTI;
    btn.listen(gpio::SignalEdge::Falling, &mut exti);

    let mut led = gpioa.pa5.into_push_pull_output();


    let mut cs = gpiob.pb0.into_push_pull_output(); // blue 13
    cs.set_high().unwrap();

    // d/c pin for toggling between data/cmd
    // dc low = command, dc high = data
    let mut dc = gpioa.pa7.into_push_pull_output(); // orange
    dc.set_low().unwrap();

    let mut rst = gpioa.pa6.into_push_pull_output();
    rst.set_high().unwrap();


    let mut usart = dp
    .USART2 // tx      // rx
    .usart(gpioa.pa2, gpioa.pa3, Config::default().baudrate(1000000.bps()), &mut rcc)
    .unwrap();

    writeln!(usart, "Hello stm32g0\n").unwrap();


    led.set_high().unwrap();
    delay.delay(500.ms());
    led.set_low().unwrap();

    let sck = gpiob.pb3; // yellow 10
    let miso = gpiob.pb4;
    let mosi = gpiob.pb5; // green 9
    let spi = dp.SPI1.spi(
        (sck, miso, mosi),
        spi::MODE_0,
        10.mhz(),
        &mut rcc);


    writeln!(usart, "Start!").unwrap();

    // do power on reset
    rst.set_low().unwrap();
    delay.delay(1.ms());
    rst.set_high().unwrap();

    // send configuration bytes
    cs.set_low().unwrap();

    writeln!(usart, "Turn on VCC!").unwrap();

    let spi_interface = ssd1362::interface::SpiInterface::new(spi, cs, dc);
    let mut display = ssd1362::display::Display::new(spi_interface, DisplayRotation::Rotate0);
    display.init().unwrap();
    display.clear(BinaryColor::Off); //.unwrap();
    // display.blank().unwrap();
    display.on().unwrap();
    display.flush().unwrap();

    // for _i in 0..64 {
    //     display.draw(&[0x00; 128]).unwrap();
    // }



    // display.write_char(&J, 0).unwrap();
    // display.write_char(&I, 1).unwrap();
    // display.write_char(&T, 2).unwrap();
    // display.write_char(&T, 3).unwrap();
    // display.write_char(&E, 4).unwrap();
    // display.write_char(&R, 5).unwrap();

    // display.write_char(&J, 7).unwrap();
    // display.write_char(&I, 8).unwrap();
    // display.write_char(&T, 9).unwrap();
    // display.write_char(&T, 10).unwrap();
    // display.write_char(&E, 11).unwrap();
    // display.write_char(&R, 12).unwrap();


    delay.delay(2000.ms());

    let c = Circle::new(Point::new(20, 20), 12).into_styled(PrimitiveStyle::with_fill(BinaryColor::On));
    let t = Text::new("Hello Rust!", Point::new(120, 16))
        .into_styled(TextStyle::new(Font6x8, BinaryColor::On));

    c.draw(&mut display);
    t.draw(&mut display);



    let style = TextStyleBuilder::new(Font24x32).background_color(BinaryColor::On).text_color(BinaryColor::Off).build();
    Text::new("YES!", Point::new(120, 30))
    .into_styled(style).draw(&mut display);

    // Rectangle::new(Point::zero(), Point::new(180,64))
    //         .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
    //         .draw(&mut display);

    display.flush().unwrap();


    let white = Rectangle::new(Point::zero(), Point::new(100,60))
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On));

    let black = Rectangle::new(Point::zero(), Point::new(100,60))
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off));

    // for i in 0..64 {
    //     display.draw(&[0x22; 64]).unwrap();
    // }
    // let mut builder = Builder::new();
    // let mut display: GraphicsMode<_> = builder.connect_spi(spi, dc).into();
    // let mut display2 = builder.connect_dummy();

    // let mut display: GraphicsMode<_> = Builder::new().connect_spi(spi, dc).into();

    // display.reset(&mut rst, &mut delay).unwrap();
    // display.init().unwrap();
    // display.clear();
    // for j in 0..64 {
    //     for i in 0..128/2 {
    //         display.set_pixel(j,i,0);
    //     }
    // }
    // display.flush().unwrap();
    // display.display_on(true).unwrap();


    // for _i in 0..(256*64/8) {
    //     spi_interface.send_data(&[0, 0, 0, 0]).unwrap();
    //     // spi_interface.send_data(&[0xDc, 0xA8, 0x64, 0x20]).unwrap();
    // }

    // // display on
    // Command::DisplayOn(true).send(&mut spi_interface).unwrap();
    // spi_interface.send_data(&[0x0F, 0x0F]).unwrap();
    // Command::ColumnAddress(0, 127).send(&mut spi_interface).unwrap();

    // for _i in 0..64 {
    //     spi_interface.send_data(&[0xFF]).unwrap();
    // }

    // Command::ColumnAddress(64, 127).send(&mut spi_interface).unwrap();
    // Command::RowAddress(1, 63).send(&mut spi_interface).unwrap();

    // for _i in 0..64 {
    //     spi_interface.send_data(&[0xFF]).unwrap();
    // }

    let mut scrollOffset = 0u8;

    loop {

        if exti.is_pending(Event::GPIO13, gpio::SignalEdge::Falling) {
            led.toggle().unwrap();
            exti.unpend(Event::GPIO13);


            writeln!(usart, "Werkt nog").unwrap();

            display.draw(&[0x0F; 1]).unwrap();
            display.draw(&[0xF0; 1]).unwrap();


        }

        // Circle::new(Point::new(20, 20), 12).into_styled(PrimitiveStyle::with_fill(BinaryColor::Off)).draw(&mut display);
        // display.flush().unwrap();

        // delay.delay(10.ms());

        // Circle::new(Point::new(20, 20), 12).into_styled(PrimitiveStyle::with_fill(BinaryColor::On)).draw(&mut display);
        // display.flush().unwrap();

        delay.delay(50.ms());

        display.scroll(scrollOffset).unwrap();
        scrollOffset += 8;

        if scrollOffset > 63 {
            scrollOffset = 0;
        }


        // white.draw(&mut display);

        // delay.delay(1860.ms());

        // black.draw(&mut display);


    }
}
