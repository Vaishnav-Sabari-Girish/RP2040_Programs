#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_alloc::LlffHeap as Heap;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use panic_halt as _;
use rp2040_hal::fugit::RateExtU32;
use rp2040_hal::{
    Sio,
    clocks::{Clock, init_clocks_and_plls},
    gpio::{FunctionSio, FunctionSpi, Pin, PullDown, SioOutput},
    pac,
    spi::Spi,
    timer::Timer,
    watchdog::Watchdog,
};

use mipidsi::{
    Builder,
    interface::SpiInterface,
    models::ST7789,
    options::{Orientation, Rotation},
};

use mousefood::prelude::*;
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::{Frame, Terminal};

#[global_allocator]
static HEAP: Heap = Heap::empty();
extern crate alloc;

// --- THE IGNITION KEY ---
#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;
// ------------------------

// Scratch buffer that mipidsi's SpiInterface uses for bulk writes.
// Must outlive the display, hence a `static`.
const DI_BUFFER_LEN: usize = 512;
static mut DI_BUFFER: [u8; DI_BUFFER_LEN] = [0u8; DI_BUFFER_LEN];

#[entry]
fn main() -> ! {
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 100_000;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe {
            let heap_ptr = core::ptr::addr_of_mut!(HEAP_MEM) as usize;
            HEAP.init(heap_ptr, HEAP_SIZE);
        }
    }

    let mut pac = pac::Peripherals::take().unwrap();
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

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let sio = Sio::new(pac.SIO);
    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // SPI0 — same bus as your e-paper setup
    let sclk = pins.gpio2.into_function::<FunctionSpi>();
    let mosi = pins.gpio3.into_function::<FunctionSpi>();

    let spi = Spi::<_, _, _, 8>::new(pac.SPI0, (mosi, sclk)).init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        32_000_000.Hz(), // TFT is happy at 32 MHz (EPD was limited to 4)
        embedded_hal::spi::MODE_0,
    );

    let cs: Pin<_, FunctionSio<SioOutput>, PullDown> = pins.gpio5.into_push_pull_output();
    let dc: Pin<_, FunctionSio<SioOutput>, PullDown> = pins.gpio6.into_push_pull_output();
    let rst: Pin<_, FunctionSio<SioOutput>, PullDown> = pins.gpio10.into_push_pull_output();
    let mut bl: Pin<_, FunctionSio<SioOutput>, PullDown> = pins.gpio11.into_push_pull_output();
    bl.set_high().unwrap(); // backlight on (ST7789 breakout; skip if your panel is always-on)

    let spi = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    // mipidsi needs a scratch buffer for bulk pixel transfers.
    let di = unsafe {
        let buf_ptr = core::ptr::addr_of_mut!(DI_BUFFER) as *mut u8;
        let buf = core::slice::from_raw_parts_mut(buf_ptr, DI_BUFFER_LEN);
        SpiInterface::new(spi, dc, buf)
    };

    // No DisplayAdapter needed — mipidsi::Display implements DrawTarget for Rgb565.
    let mut display = Builder::new(ST7789, di)
        .display_size(240, 320)
        .orientation(Orientation::new().rotate(Rotation::Deg90)) // landscape 320x240
        .reset_pin(rst)
        .init(&mut timer)
        .unwrap();

    display.clear(Rgb565::BLACK).unwrap();

    // No flush_callback — the ST7789 GRAM accepts writes directly.
    let backend = EmbeddedBackend::new(&mut display, EmbeddedBackendConfig::default());
    let mut terminal = Terminal::new(backend).unwrap();

    // E-paper drew once and slept. TFT redraws continuously.
    loop {
        terminal.draw(draw).unwrap();
        timer.delay_ms(500);
    }
}

fn draw(frame: &mut Frame) {
    let text = "Ratatui on TFT";

    let theme = Style::default()
        .fg(ratatui::style::Color::White)
        .bg(ratatui::style::Color::Black);

    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });
    let block = Block::bordered().title("Mousefood").style(theme);
    frame.render_widget(paragraph.block(block), frame.area());
}
