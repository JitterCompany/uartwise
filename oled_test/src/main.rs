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
    exti::Event
};

use nb::block;

mod interface;
use crate::interface::SpiInterface;
use crate::interface::DisplayInterface;


mod command;
use crate::command::{Command, DisplayMode, VcomhLevel};

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("cannot take peripherals");
    let mut rcc = dp.RCC.constrain();

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
        1000.khz(),
        &mut rcc);


    writeln!(usart, "Start!").unwrap();

    // do power on reset
    rst.set_low().unwrap();
    delay.delay(1.ms());
    rst.set_high().unwrap();

    // send configuration bytes
    cs.set_low().unwrap();

    writeln!(usart, "Turn on VCC!").unwrap();

    let mut spi_interface = SpiInterface::new(spi, dc);
    Command::InternalVDD(true).send(&mut spi_interface).unwrap();
    Command::InternalIREF(true).send(&mut spi_interface).unwrap();
    Command::ColumnAddress(0, 0x7f).send(&mut spi_interface).unwrap();
    Command::RowAddress(0, 0x3f).send(&mut spi_interface).unwrap();

    Command::Remap(0x43).send(&mut spi_interface).unwrap();
    Command::StartLine(0).send(&mut spi_interface).unwrap();
    Command::DisplayOffset(0).send(&mut spi_interface).unwrap();
    Command::Mode(DisplayMode::Normal).send(&mut spi_interface).unwrap();
    Command::Multiplex(0x3F).send(&mut spi_interface).unwrap();
    Command::PhaseLength(0x11).send(&mut spi_interface).unwrap();
    Command::DisplayClockDiv(0xF, 0x0).send(&mut spi_interface).unwrap();
    Command::DefaultGrayScale().send(&mut spi_interface).unwrap();
    Command::PreChargeVoltage(0x04).send(&mut spi_interface).unwrap();
    Command::VcomhDeselect(VcomhLevel::V082).send(&mut spi_interface).unwrap();


    for _i in 0..(256*64/8) {

        spi_interface.send_data(&[0xDc, 0xA8, 0x64, 0x20]).unwrap();
    }

    loop {

        if exti.is_pending(Event::GPIO13, gpio::SignalEdge::Falling) {
            led.toggle().unwrap();
            exti.unpend(Event::GPIO13);



            // display on
            Command::DisplayOn(true).send(&mut spi_interface).unwrap();



        }



    }
}
