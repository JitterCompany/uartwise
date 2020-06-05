#![no_std]
#![no_main]

use core::fmt::Write;

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use cortex_m_rt::entry;

use display_interface_spi::SPIInterface;

use embedded_hal as hal;
use hal::digital::v2::OutputPin;

use stm32g0xx_hal::{
    prelude::*,
    stm32::{self, SPI1},
    spi::{self, Spi},
    serial::Config,
    gpio,
    exti::Event,
    rcc
};

use ssd1362::{self, display::DisplayRotation, terminal, Font6x8};

type Terminal = terminal::TerminalView<SPIInterface<
    spi::Spi<
        SPI1,
        (
            gpio::gpiob::PB3<gpio::Input<gpio::Floating>>,
            gpio::gpioa::PA6<gpio::Input<gpio::Floating>>,
            gpio::gpiob::PB5<gpio::Input<gpio::Floating>>
        )
    >,
    gpio::gpiob::PB7<gpio::Output<gpio::PushPull>>,
    gpio::gpiob::PB4<gpio::Output<gpio::PushPull>>
    >,
    Font6x8
    >;

#[rtfm::app(device = stm32g0xx_hal::stm32)]
const APP: () = {

    struct Resources {
        terminal: Terminal
    }

    #[init(spawn = [startup])]
    fn init(cx: init::Context) -> init::LateResources {

        let dp = stm32::Peripherals::take().expect("cannot take peripherals");

        let pll_cfg = rcc::PllConfig::default();
        let rcc_cfg = rcc::Config::pll().pll_cfg(pll_cfg);
        let mut rcc = dp.RCC.freeze(rcc_cfg);

        let mut delay = dp.TIM15.delay(&mut rcc);

        let gpioa = dp.GPIOA.split(&mut rcc);
        let gpiob = dp.GPIOB.split(&mut rcc);
        let _gpioc = dp.GPIOC.split(&mut rcc);



        // let mut led = gpioa.pa5.into_push_pull_output();
        let mut led_g = gpioa.pa7.into_push_pull_output();
        let mut led_r = gpiob.pb0.into_push_pull_output();
        let mut en_16v = gpioa.pa1.into_push_pull_output();

        let encoder_a = gpiob.pb1.into_floating_input();
        let encoder_b = gpiob.pb2.into_floating_input();
        let btn = gpioa.pa8.into_pull_up_input();
        let mut exti = dp.EXTI;
        btn.listen(gpio::SignalEdge::Falling, &mut exti);

        let mut cs = gpiob.pb4.into_push_pull_output(); // blue 13
        cs.set_high().unwrap();

        // d/c pin for toggling between data/cmd
        // dc low = command, dc high = data
        let mut dc = gpiob.pb7.into_push_pull_output(); // orange
        dc.set_low().unwrap();

        let mut rst = gpiob.pb6.into_push_pull_output();
        rst.set_high().unwrap();

        let mut usart = dp
        .USART1 // tx      // rx
        .usart(gpioa.pa9, gpioa.pa10, Config::default().baudrate(115200.bps()), &mut rcc)
        .unwrap();

        writeln!(usart, "Hello SerialLogger\n").unwrap();

        led_g.set_high().unwrap();
        delay.delay(500.ms());
        led_g.set_low().unwrap();

        let sck = gpiob.pb3; // yellow 10
        let miso = gpioa.pa6; //not used
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


        writeln!(usart, "Turn on VCC!").unwrap();
        en_16v.set_high().unwrap();

        let interface = display_interface_spi::SPIInterface::new(spi, dc, cs);
        let display = ssd1362::display::Display::new(interface, DisplayRotation::Rotate180);
        writeln!(usart, "create terminal..").unwrap();
        let font = terminal::Font6x8 {};
        let mut terminal = terminal::TerminalView::new(display, font);
        terminal.init().unwrap();

        writeln!(usart, "Display init done!").unwrap();
        writeln!(terminal, "Display init done!").unwrap();

        cx.spawn.startup().ok();

        init::LateResources {
            terminal
        }
    }

    #[task(resources=[terminal])]
    fn startup(cx: startup::Context) {
        let startup::Resources {
            terminal,
        } = cx.resources;

        writeln!(terminal, "info: Still working").unwrap();

    }

     // Interrupt handlers used to dispatch software tasks
     extern "C" {
        fn SPI2();
    }
};
