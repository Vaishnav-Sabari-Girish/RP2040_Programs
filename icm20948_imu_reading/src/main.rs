#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::delay::DelayNs;
use icm20948::{I2cInterface, Icm20948Driver, MagConfig};
use panic_halt as _;
use rp2040_hal::fugit::RateExtU32;
use rp2040_hal::{
    I2C, Sio,
    clocks::{Clock, init_clocks_and_plls},
    gpio::FunctionI2C,
    pac,
    timer::Timer,
    usb::UsbBus,
    watchdog::Watchdog,
};
use usb_device::{class_prelude::UsbBusAllocator, device::StringDescriptors, prelude::*};

use usbd_serial::{SerialPort, USB_CLASS_CDC};

// --- THE IGNITION KEY ---
// This places the 256-byte bootloader at the very start of the flash memory.
// Without this, the RP2040 ROM refuses to jump to our code.
#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;
// ------------------------

struct SpinDelay<'a> {
    timer: &'a Timer,
}

impl<'a> SpinDelay<'a> {
    fn new(timer: &'a Timer) -> Self {
        Self { timer }
    }
}

impl embedded_hal::delay::DelayNs for SpinDelay<'_> {
    fn delay_ns(&mut self, ns: u32) {
        let start = self.timer.get_counter().ticks();
        let target = start.saturating_add((ns as u64) / 1000);
        while self.timer.get_counter().ticks() < target {}
    }
}

fn usb_log<B: usb_device::class_prelude::UsbBus>(serial: &mut SerialPort<'_, B>, msg: &str) {
    let _ = serial.write(msg.as_bytes());
    let _ = serial.write(b"\r\n");
}

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    // let cp = pac::CorePeripherals::take().unwrap();

    let mut wdt = Watchdog::new(pac.WATCHDOG);
    let clocks = init_clocks_and_plls(
        12_000_000u32,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut wdt,
    )
    .ok()
    .unwrap();

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let sio = Sio::new(pac.SIO);
    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let mut serial = SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("Rat Company")
            .product("ICM-20948 Logger")
            .serial_number("TEST")])
        .unwrap()
        .device_class(USB_CLASS_CDC)
        .build();

    let sda = pins
        .gpio6
        .into_pull_up_input()
        .into_function::<FunctionI2C>();

    let scl = pins
        .gpio7
        .into_pull_up_input()
        .into_function::<FunctionI2C>();

    let i2c = I2C::i2c1(
        pac.I2C1,
        sda,
        scl,
        400.kHz(),
        &mut pac.RESETS,
        clocks.system_clock.freq(),
    );

    let i2c_interface = I2cInterface::default(i2c);
    let mut imu = Icm20948Driver::try_new(i2c_interface).expect("ICM-20948 not found at 0x68");

    let mut delay = SpinDelay::new(&timer);

    imu.init(&mut delay).expect("ICM-20948 init failed");

    // Magnetometer is initialized separately
    let mag_config = MagConfig {
        mode: icm20948::MagMode::Continuous100Hz,
    };

    imu.init_magnetometer(mag_config, &mut delay)
        .expect("Magnetometer init failed");

    delay.delay_ms(100);

    let mut last_sensor_read = timer.get_counter().ticks();

    loop {
        usb_dev.poll(&mut [&mut serial]);

        let now = timer.get_counter().ticks();

        if now.wrapping_sub(last_sensor_read) >= 1_000_000 {
            last_sensor_read = now;
            if let Ok(accel) = imu.read_accelerometer() {
                let mut msg = heapless::String::<64>::new();

                let _ = core::fmt::write(
                    &mut msg,
                    format_args!("Accel: X={:.2} Y={:.2} Z={:.2}", accel.x, accel.y, accel.z),
                );
                usb_log(&mut serial, &msg);
            }

            if let Ok(gyro) = imu.read_gyroscope() {
                let mut msg = heapless::String::<64>::new();

                let _ = core::fmt::write(
                    &mut msg,
                    format_args!("Gyro: X={:.2} Y={:.2} Z={:.2}", gyro.x, gyro.y, gyro.z),
                );
                usb_log(&mut serial, &msg);
            }
            if let Ok(mag) = imu.read_magnetometer() {
                let mut msg = heapless::String::<64>::new();

                let _ = core::fmt::write(
                    &mut msg,
                    format_args!("Mag: X={:.2} Y={:.2} Z={:.2}", mag.x, mag.y, mag.z),
                );
                usb_log(&mut serial, &msg);
            }
            if let Ok(temp) = imu.read_temperature_celsius() {
                let mut msg = heapless::String::<32>::new();

                let _ = core::fmt::write(&mut msg, format_args!("Temp: {:.1}C", temp));
                usb_log(&mut serial, &msg);
            }
        }
    }
}
