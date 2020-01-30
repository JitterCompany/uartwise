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
    rcc,
    gpio,
    exti::Event

};

use nb::block;


#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("cannot take peripherals");
    let mut rcc = dp.RCC.constrain(); //freeze(rcc::Config::lsi());

    let mut delay = dp.TIM15.delay(&mut rcc);

    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);
    let gpioc = dp.GPIOC.split(&mut rcc);

    let mut btn = gpioc.pc13.into_pull_up_input();
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



    // let mut sck = gpioa.pa1;
    // // sck.set_alt_mode(gpio::AltFunction::AF0);
    // let miso = gpioa.pa6;
    // let mosi1 = gpioa.pa7;
    let sck = gpiob.pb3; // yellow 10
    let miso = gpiob.pb4;
    let mosi = gpiob.pb5; // green 9
    let mut spi = dp.SPI1.spi(
        (sck, miso, mosi),
        spi::MODE_0,
        1000.khz(),
        &mut rcc);

    // cs.set_low().unwrap();
    // block!(spi.send(0x01)).unwrap();
    // block!(spi.read()).unwrap();
    // block!(spi.send(0x0)).unwrap();
    // let res = block!(spi.read());
    // cs.set_high().unwrap();

    // match res {
    //     Ok(b) => {
    //         writeln!(usart, "Got byte {}", b).unwrap();

    //         if b == 0xAD {
    //             led.set_high().unwrap();
    //         }
    //     }
    //     Err(e) => {
    //         writeln!(usart, "Error: {:?}", e).unwrap();
    //     },
    // }


    writeln!(usart, "Start!").unwrap();

    // do power on reset
    rst.set_low().unwrap();
    delay.delay(1.ms());
    rst.set_high().unwrap();

    // send configuration bytes
    dc.set_low().unwrap();
    cs.set_low().unwrap();

    // // Set Vdd Mode VCI = 3.3V
    block!(spi.send(0xAB)).unwrap();
    block!(spi.read()).unwrap();
    block!(spi.send(0x01)).unwrap();
    block!(spi.read()).unwrap();

    // // Set IREF selection
    block!(spi.send(0xAD)).unwrap();
    block!(spi.read()).unwrap();
    block!(spi.send(0x9E)).unwrap();
    block!(spi.read()).unwrap();

    // Set column address
    block!(spi.send(0x15)).unwrap();
    block!(spi.read()).unwrap();
    block!(spi.send(0x00)).unwrap();
    block!(spi.read()).unwrap();
    block!(spi.send(0x7f)).unwrap();
    block!(spi.read()).unwrap();


    // // Set row address
    block!(spi.send(0x75)).unwrap();
    block!(spi.read()).unwrap();
    block!(spi.send(0x00)).unwrap();
    block!(spi.read()).unwrap();
    block!(spi.send(0x3f)).unwrap();
    block!(spi.read()).unwrap();


    // // Set segment re-map
    block!(spi.send(0xA0)).unwrap();
    block!(spi.read()).unwrap();
    block!(spi.send(0x43)).unwrap();
    block!(spi.read()).unwrap();

    // Set display start line
    block!(spi.send(0xA1)).unwrap();
    block!(spi.read()).unwrap();
    block!(spi.send(0x00)).unwrap();
    block!(spi.read()).unwrap();

    // // Set display offset
    block!(spi.send(0xA2)).unwrap();
    block!(spi.read()).unwrap();
    block!(spi.send(0x00)).unwrap();
    block!(spi.read()).unwrap();

    // // Set normal display mode
    block!(spi.send(0xA4)).unwrap();
    block!(spi.read()).unwrap();

    // // Set multiplex ratio
    block!(spi.send(0xA8)).unwrap();
    block!(spi.read()).unwrap();
    block!(spi.send(0x3F)).unwrap();
    block!(spi.read()).unwrap();

    // // Set Phase1,2 length
    block!(spi.send(0xB1)).unwrap();
    block!(spi.read()).unwrap();
    block!(spi.send(0x11)).unwrap();
    block!(spi.read()).unwrap();

    // // Set display clock divide ratio
    block!(spi.send(0xB3)).unwrap();
    block!(spi.read()).unwrap();
    block!(spi.send(0xF0)).unwrap();
    block!(spi.read()).unwrap();

    // // Grey scale table
    block!(spi.send(0xB9)).unwrap();
    block!(spi.read()).unwrap();

    // // Set pre-charge voltage
    block!(spi.send(0xBC)).unwrap();
    block!(spi.read()).unwrap();
    block!(spi.send(0x04)).unwrap();
    block!(spi.read()).unwrap();

    // // Set VCOMH deselect level, 0.82 * Vcc
    block!(spi.send(0xBE)).unwrap();
    block!(spi.read()).unwrap();
    block!(spi.send(0x05)).unwrap();
    block!(spi.read()).unwrap();

    cs.set_high().unwrap();

    writeln!(usart, "Turn on VCC!").unwrap();




    loop {


        // delay.delay(500.ms());

        // led.set_high().unwrap();

        // delay.delay(500.ms());

        // led.set_low().unwrap();

        if exti.is_pending(Event::GPIO13, gpio::SignalEdge::Falling) {
            led.toggle().unwrap();
            exti.unpend(Event::GPIO13);

            // display on
            cs.set_low().unwrap();

            block!(spi.send(0xAF)).unwrap();
            block!(spi.read()).unwrap();


            delay.delay(1.ms());
            dc.set_high().unwrap();

            for _i in 0..(256*64/8) {

                block!(spi.send(0xFF)).unwrap();
                block!(spi.read()).unwrap();
                block!(spi.send(0xFF)).unwrap();
                block!(spi.read()).unwrap();
                block!(spi.send(0x00)).unwrap();
                block!(spi.read()).unwrap();
                block!(spi.send(0x00)).unwrap();
                block!(spi.read()).unwrap();
            }

            cs.set_high().unwrap();

        }



    }
}


// #[interrupt]
// fn EXTI0_1() {

//     ctx.resources.exti.unpend(Event::GPIO1);

// }