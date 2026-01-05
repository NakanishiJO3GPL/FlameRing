//! Flame Ring Raspberry Pi Pico 2
//!
#![no_std]
#![no_main]

mod button;
mod proximity;
mod state_machine;
mod animation;

use core::time as core_time;
use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::{Input, Pull},
    i2c::{self, Config, I2c},
    peripherals::{I2C0, PIO0},
    pio::{self, Pio},
    pio_programs::pwm::{PioPwm, PioPwmProgram},
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});

// Global channel (4 messages)
pub static CHANNEL: Channel<CriticalSectionRawMutex, Event, 4> = Channel::new();

// Message
pub enum ButtonKind {
    Nikomi,
    Weak,
    Strong,
    Power,
}

pub enum Event {
    ProximityChanged(u16),
    ProximityCurrent(u16),
    ButtonPressed(ButtonKind),
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Flame Ring Raspberry Pi Pico 2 example");
    let p = embassy_rp::init(Default::default());
    let Pio {
        mut common,
        sm0,
        sm1,
        ..
    } = Pio::new(p.PIO0, Irqs);

    let prog = PioPwmProgram::new(&mut common);
    let mut pwm_pio0 = PioPwm::new(&mut common, sm0, p.PIN_4, &prog);
    let mut pwm_pio1 = PioPwm::new(&mut common, sm1, p.PIN_5, &prog);
    pwm_pio0.set_period(core_time::Duration::from_millis(10));
    pwm_pio0.start();
    pwm_pio1.set_period(core_time::Duration::from_millis(10));
    pwm_pio1.start();

    let i2c = I2c::new_async(
        p.I2C0,
        p.PIN_13, // SCL
        p.PIN_12, // SDA
        Irqs,
        Config::default(),
    );

    let button_nikomi = Input::new(p.PIN_6, Pull::Up); // Nikomi
    let button_weak = Input::new(p.PIN_7, Pull::Up); // Weak
    let button_strong = Input::new(p.PIN_8, Pull::Up); // Strong
    let button_power = Input::new(p.PIN_9, Pull::Up); // Power

    // Spawn tasks
    spawner
        .spawn(proximity::proximity_sensor_task(i2c))
        .unwrap();
    spawner
        .spawn(state_machine::animation_state_task(pwm_pio0, pwm_pio1))
        .unwrap();
    spawner
        .spawn(button::button_nikomi_handler_task(button_nikomi))
        .unwrap();
    spawner
        .spawn(button::button_weak_handler_task(button_weak))
        .unwrap();
    spawner
        .spawn(button::button_strong_handler_task(button_strong))
        .unwrap();
    spawner
        .spawn(button::button_power_handler_task(button_power))
        .unwrap();
}

// End of file
