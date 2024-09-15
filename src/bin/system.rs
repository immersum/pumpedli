#![no_std]
#![no_main]

mod resources;

use core::cell::RefCell;
use core::ptr::addr_of_mut;

use defmt::{panic, unwrap, Format};
use defmt_rtt as _;
use display_interface_spi::SPIInterface as SpiInterface;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::{Executor, Spawner};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{AnyPin, Input, Level, Output, Pull};
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::peripherals::{I2C1, PIO0, SPI1, USB};
use embassy_rp::{i2c, pio, spi, usb};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::blocking_mutex::NoopMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::pubsub::PubSubChannel;
use embassy_sync::signal::Signal;
use embassy_time::{Delay, Duration};
use embedded_hal::spi::SpiDevice;
use embedded_hal_async::i2c::I2c;
use mipidsi::models::GC9A01;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use mipidsi::Builder;
use panic_probe as _;
use pumpedli::control::ActionPubSubChannel;
use pumpedli::dev::ads1115::{Addr, Ads1115};
use pumpedli::dev::cd4067::Cd4067;
use pumpedli::dev::ws2812::Ws2812;
use pumpedli::reading::ReadingPubSubChannel;
use pumpedli::{adc, control, display, led, program, rgb};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});

type I2cDriver = i2c::I2c<'static, I2C1, i2c::Async>;
type SpiDriver = spi::Spi<'static, SPI1, spi::Blocking>;

#[embassy_executor::task]
async fn led_task(blinker: led::Blinker<'static>) -> ! {
    blinker.run().await
}

#[embassy_executor::task]
async fn rgb_task(control: rgb::Control<'static, PIO0, 0>) -> ! {
    control.run().await
}

#[embassy_executor::task]
async fn logger_task(driver: usb::Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::task]
async fn i2c_spawner_task(
    spawner: Spawner,
    i2c_bus: &'static mut Mutex<NoopRawMutex, impl I2c<Error: Format> + 'static>,
    adc_rdy_pins: [(Addr, AnyPin); 4],
    control_loops: [&'static control::Loop<'_>; 16],
    reading_bus: &'static ReadingPubSubChannel<'_>,
) {
    let iter = adc_rdy_pins.into_iter();
    let zip = iter.zip(control_loops.chunks(4));

    for ((addr, rdy_pin), control_loops) in zip {
        let i2c_dev = I2cDevice::new(i2c_bus);
        let rdy = Input::new(rdy_pin, Pull::Up);
        let ads1115 = unwrap!(Ads1115::new(i2c_dev, addr, rdy).await);
        let control_loops = unwrap!(control_loops.try_into());
        let publisher = unwrap!(reading_bus.publisher());
        let converter = adc::Converter::new(ads1115, control_loops, publisher);
        unwrap!(spawner.spawn(adc_task(converter)));
    }

    for control_loop in control_loops.iter().take(9) {
        let mut program = control_loop.program.lock().await;
        program.replace(Default::default());
    }
}

#[embassy_executor::task(pool_size = 4)]
async fn adc_task(converter: adc::Converter<'static, impl I2c + 'static>) -> ! {
    converter.run().await
}

#[embassy_executor::task]
async fn display_task(mut dashboard: display::Dashboard<'static, impl SpiDevice + 'static>) -> ! {
    dashboard.run().await
}

#[embassy_executor::task]
async fn action_spawner_task(
    spawner: Spawner,
    action_bus: &'static ActionPubSubChannel<'_>,
    control_loops: [&'static control::Loop<'_>; 16],
    control_mutex: &'static Mutex<NoopRawMutex, (Output<'_>, Cd4067<'_>)>,
    led: &'static Signal<CriticalSectionRawMutex, led::Mode>,
    rgb: &'static Signal<CriticalSectionRawMutex, rgb::Mode>,
) {
    for &control::Loop { mux_output, .. } in control_loops {
        let subscriber = unwrap!(action_bus.subscriber());
        let irrigator = control::Irrigator::new(subscriber, mux_output, control_mutex, led, rgb);
        unwrap!(spawner.spawn(control_task(irrigator)));
    }
}

#[embassy_executor::task(pool_size = 16)]
async fn control_task(mut irrigator: control::Irrigator<'static>) -> ! {
    irrigator.run().await
}

#[embassy_executor::task]
async fn program_task(mut regulator: program::Regulator<'static>) -> ! {
    regulator.run().await
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    let driver = usb::Driver::new(p.USB, Irqs);

    static LED_SIGNAL: Signal<CriticalSectionRawMutex, led::Mode> = Signal::new();
    let led = Output::new(AnyPin::from(p.PIN_25), Level::Low);
    let blinker = led::Blinker::new(&LED_SIGNAL, led);

    static LED_RGB_SIGNAL: Signal<CriticalSectionRawMutex, rgb::Mode> = Signal::new();
    let mut pio = pio::Pio::new(p.PIO0, Irqs);
    let ws2812 = Ws2812::new(&mut pio.common, pio.sm0, p.DMA_CH0, p.PIN_23);
    let control = rgb::Control::new(&LED_RGB_SIGNAL, ws2812);

    static I2C_BUS: StaticCell<Mutex<NoopRawMutex, I2cDriver>> = StaticCell::new();
    let i2c = i2c::I2c::new_async(p.I2C1, p.PIN_3, p.PIN_2, Irqs, i2c::Config::default());
    let i2c_bus = I2C_BUS.init(Mutex::new(i2c));
    let adc_rdy_pins = [
        (Addr::Gnd, AnyPin::from(p.PIN_4)),
        (Addr::Vdd, AnyPin::from(p.PIN_5)),
        (Addr::Sda, AnyPin::from(p.PIN_6)),
        (Addr::Scl, AnyPin::from(p.PIN_7)),
    ];

    let control_loops = resources::control::LOOPS.each_ref();

    static SPI_BUS: StaticCell<NoopMutex<RefCell<SpiDriver>>> = StaticCell::new();
    let spi = spi::Spi::new_blocking(p.SPI1, p.PIN_10, p.PIN_11, p.PIN_8, Default::default());
    let spi_bus = SPI_BUS.init(NoopMutex::new(RefCell::new(spi)));
    let cs = Output::new(AnyPin::from(p.PIN_9), Level::High);
    let dc = Output::new(AnyPin::from(p.PIN_13), Level::Low);
    let rst = Output::new(AnyPin::from(p.PIN_12), Level::High);
    let _blk = Output::new(AnyPin::from(p.PIN_14), Level::High);

    static ACTION_BUS: control::ActionPubSubChannel = PubSubChannel::new();
    let motor = Output::new(AnyPin::from(p.PIN_15), Level::High);
    let en = Output::new(AnyPin::from(p.PIN_16), Level::High);
    let s0 = Output::new(AnyPin::from(p.PIN_17), Level::Low);
    let s1 = Output::new(AnyPin::from(p.PIN_18), Level::Low);
    let s2 = Output::new(AnyPin::from(p.PIN_19), Level::Low);
    let s3 = Output::new(AnyPin::from(p.PIN_20), Level::Low);
    let cd4067 = Cd4067::new(en, s0, s1, s2, s3);

    static CONTROL_MUTEX: StaticCell<Mutex<NoopRawMutex, (Output, Cd4067)>> = StaticCell::new();
    let control_mutex = CONTROL_MUTEX.init(Mutex::new((motor, cd4067)));

    static READING_BUS: ReadingPubSubChannel = PubSubChannel::new();
    let subscriber = unwrap!(READING_BUS.subscriber());
    let publisher = unwrap!(ACTION_BUS.publisher());
    let regulator = program::Regulator::new(subscriber, publisher, &LED_SIGNAL, &LED_RGB_SIGNAL);

    static mut CORE1_STACK: Stack<8192> = Stack::new();
    let subscriber = unwrap!(READING_BUS.subscriber());

    spawn_core1(
        p.CORE1,
        unsafe { &mut *addr_of_mut!(CORE1_STACK) },
        move || {
            let mut config = spi::Config::default();
            config.frequency = 16_000_000;

            let spi_dev = SpiDeviceWithConfig::new(spi_bus, cs, config);
            let spi_int = SpiInterface::new(spi_dev, dc);
            let builder = Builder::new(GC9A01, spi_int)
                .reset_pin(rst)
                .color_order(ColorOrder::Bgr)
                .invert_colors(ColorInversion::Inverted)
                .orientation(Orientation {
                    rotation: Rotation::Deg90,
                    ..Default::default()
                });

            let Ok(display) = builder.init(&mut Delay) else {
                panic!("init failed");
            };

            let dashboard = display::Dashboard::new(subscriber, display);

            static EXECUTOR: StaticCell<Executor> = StaticCell::new();
            let executor = EXECUTOR.init(Executor::new());

            executor.run(|spawner| unwrap!(spawner.spawn(display_task(dashboard))))
        },
    );

    LED_SIGNAL.signal(led::Mode::OnOff(
        Duration::from_millis(600),
        Duration::from_millis(2400),
    ));

    LED_RGB_SIGNAL.signal(rgb::Mode::Off);

    static EXECUTOR: StaticCell<Executor> = StaticCell::new();
    let executor = EXECUTOR.init(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(logger_task(driver)));
        unwrap!(spawner.spawn(led_task(blinker)));
        unwrap!(spawner.spawn(rgb_task(control)));

        unwrap!(spawner.spawn(i2c_spawner_task(
            spawner,
            i2c_bus,
            adc_rdy_pins,
            control_loops,
            &READING_BUS,
        )));

        unwrap!(spawner.spawn(action_spawner_task(
            spawner,
            &ACTION_BUS,
            control_loops,
            control_mutex,
            &LED_SIGNAL,
            &LED_RGB_SIGNAL,
        )));

        unwrap!(spawner.spawn(program_task(regulator)))
    })
}
