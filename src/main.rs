#![no_std]
#![no_main]

mod ctx;
mod display;
mod game;
mod gameover;
mod gfx;
mod intro;
mod pieces;
mod timer;

use crate::ctx::Context;
use defmt_rtt as _;
use eh0::timer::CountDown;
use embedded_hal::digital::InputPin;
use fugit::ExtU32;
use fugit::RateExtU32;
use panic_halt as _;
use waveshare_rp2040_zero::entry;
use waveshare_rp2040_zero::{
    Pins, XOSC_CRYSTAL_FREQ,
    hal::{
        Sio,
        clocks::{Clock, init_clocks_and_plls},
        i2c::I2C,
        pac,
        timer::Timer,
        watchdog::Watchdog,
    },
};

pub enum Action {
    Pressed,
    Released,
}

#[derive(Default)]
pub struct Input {
    on: bool,
}

impl Input {
    fn probe<F>(&mut self, f: F) -> Option<Action>
    where
        F: FnOnce() -> bool,
    {
        if f() {
            if !self.on {
                self.on = true;
                return Some(Action::Pressed);
            }
        } else if self.on {
            self.on = false;
            return Some(Action::Released);
        }
        None
    }
}

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();

    // Configure clocks and timers
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut delay = timer.count_down();

    // Configure gpio
    let sio = Sio::new(pac.SIO);
    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure display
    let i2c = I2C::i2c1(
        pac.I2C1,
        pins.gp26.into_pull_type().into_function(), // sda
        pins.gp27.into_pull_type().into_function(), // scl
        400.kHz(),
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
    );
    let mut display = display::init(i2c);

    // configure button
    let mut button_down_pin = pins.gp0.into_pull_up_input();
    let mut button_right_pin = pins.gp1.into_pull_up_input();
    let mut button_up_pin = pins.gp3.into_pull_up_input();
    let mut button_left_pin = pins.gp7.into_pull_up_input();
    let mut button_center_pin = pins.gp8.into_pull_up_input();

    let mut button_down = Input::default();
    let mut button_right = Input::default();
    let mut button_up = Input::default();
    let mut button_left = Input::default();
    let mut button_center = Input::default();

    let mut ctx = Context::new();

    // enter loop
    loop {
        match button_down.probe(|| button_down_pin.is_low().unwrap()) {
            Some(Action::Pressed) => ctx.button_down(),
            Some(Action::Released) => (),
            None => (),
        }
        match button_right.probe(|| button_right_pin.is_low().unwrap()) {
            Some(Action::Pressed) => ctx.button_right(),
            Some(Action::Released) => (),
            None => (),
        }
        match button_up.probe(|| button_up_pin.is_low().unwrap()) {
            Some(Action::Pressed) => ctx.button_up(),
            Some(Action::Released) => (),
            None => (),
        }
        match button_left.probe(|| button_left_pin.is_low().unwrap()) {
            Some(Action::Pressed) => ctx.button_left(),
            Some(Action::Released) => (),
            None => (),
        }
        match button_center.probe(|| button_center_pin.is_low().unwrap()) {
            Some(Action::Pressed) => ctx.button_center(),
            Some(Action::Released) => (),
            None => (),
        }

        ctx.tick();

        // render screen
        display.clear();
        ctx.render(&mut display);
        display.flush().unwrap();

        // sleep for frame rate
        delay.start(50.millis());
        let _ = nb::block!(delay.wait());
    }
}
