//! Animation task
//!

use core::time as core_time;
use defmt::info;
use embassy_rp::{peripherals::PIO0, pio_programs::pwm::PioPwm};
use embassy_time as em_time;
use micromath::F32Ext;

use crate::{ButtonKind, CHANNEL, Event};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum State {
    PowerOff,
    Standby,
    PowerOn,
    PanShake,
    Nikomi,
    LevelUp,
    LevelDown,
}
impl Default for State {
    fn default() -> Self {
        State::PowerOff
    }
}

const PAN_ON_TH: u16 = 1500;
const PAN_OFF_TH: u16 = 3500;
const MAX_DUTY_MICRO: u64 = 2_500;

#[embassy_executor::task]
pub async fn animation_state_task(
    mut pwm0: PioPwm<'static, PIO0, 0>,
    mut pwm1: PioPwm<'static, PIO0, 1>,
) {
    let mut state = State::default();
    let mut prev_state = state;
    let mut level: u8 = 5;
    let mut time_stamp: u16 = 0;

    loop {
        state = update_state(state, &mut level);

        // Update LED
        if state == State::Nikomi {
            // In Nikomi state, keep LED on
            nikomi_animation(time_stamp, &mut pwm0, &mut pwm1);
            time_stamp = time_stamp.wrapping_add(1);
        } else if prev_state != state {
            prev_state = state;
            match state {
                State::PowerOff => {
                    info!("LED Off");
                    power_off_animation(level, &mut pwm0, &mut pwm1).await;
                }
                State::Standby => {
                    info!("LED Standby");
                    standby_animation(level, &mut pwm0, &mut pwm1).await;
                    // Wait for welcome animation finish
                    em_time::Timer::after(em_time::Duration::from_millis(500)).await;
                }
                State::PowerOn => {
                    info!("LED PowerOn");
                    power_on_animation(level, &mut pwm0, &mut pwm1).await;
                }
                State::PanShake => {
                    info!("LED PanShake");
                    pan_shake_animation(level, &mut pwm0, &mut pwm1).await;
                }
                State::LevelDown => {
                    info!("LED Level Down to {}", level);
                    level_change_animation(level, &mut pwm0, &mut pwm1).await;
                }
                State::LevelUp => {
                    info!("LED Level Up to {}", level);
                    level_change_animation(level, &mut pwm0, &mut pwm1).await;
                }
                _ => {
                    // Do nothing
                }
            }
        }

        em_time::Timer::after(em_time::Duration::from_millis(10)).await;
    }
}

fn update_state(state: State, level: &mut u8) -> State {
    match state {
        State::PowerOff => match CHANNEL.try_receive() {
            Ok(Event::ButtonPressed(ButtonKind::Power)) => State::Standby,
            _ => State::PowerOff,
        },
        State::Standby => match CHANNEL.try_receive() {
            Ok(Event::ButtonPressed(ButtonKind::Power)) => State::PowerOff,
            Ok(Event::ProximityCurrent(p)) => {
                if p < PAN_ON_TH {
                    State::PowerOn
                } else {
                    State::Standby
                }
            }
            Ok(Event::ProximityChanged(p)) => {
                if p < PAN_ON_TH {
                    State::PowerOn
                } else {
                    State::Standby
                }
            }
            _ => State::Standby,
        },
        State::PowerOn => match CHANNEL.try_receive() {
            Ok(Event::ButtonPressed(ButtonKind::Power)) => State::PowerOff,
            Ok(Event::ButtonPressed(ButtonKind::Weak)) => {
                if *level > 0 {
                    *level -= 1;
                    State::LevelDown
                } else {
                    State::PowerOn
                }
            }
            Ok(Event::ButtonPressed(ButtonKind::Strong)) => {
                if *level < 9 {
                    *level += 1;
                    State::LevelUp
                } else {
                    State::PowerOn
                }
            }
            Ok(Event::ButtonPressed(ButtonKind::Nikomi)) => {
                *level = 5;
                State::Nikomi
            }
            Ok(Event::ProximityChanged(p)) => {
                if p > PAN_OFF_TH {
                    State::Standby
                } else {
                    State::PanShake
                }
            }
            _ => State::PowerOn,
        },
        State::PanShake => match CHANNEL.try_receive() {
            Ok(Event::ButtonPressed(ButtonKind::Power)) => State::PowerOff,
            Ok(Event::ButtonPressed(ButtonKind::Weak)) => {
                if *level > 0 {
                    *level -= 1;
                    State::LevelDown
                } else {
                    State::PowerOn
                }
            }
            Ok(Event::ButtonPressed(ButtonKind::Strong)) => {
                if *level < 9 {
                    *level += 1;
                    State::LevelUp
                } else {
                    State::PowerOn
                }
            }
            Ok(Event::ProximityChanged(p)) => {
                if p > PAN_OFF_TH {
                    State::Standby
                } else {
                    State::PowerOn
                }
            }
            _ => State::PowerOn,
        },
        State::Nikomi => match CHANNEL.try_receive() {
            Ok(Event::ButtonPressed(ButtonKind::Power)) => State::PowerOff,
            Ok(Event::ButtonPressed(ButtonKind::Weak)) => {
                if *level > 0 {
                    *level -= 1;
                    State::LevelDown
                } else {
                    State::PowerOn
                }
            }
            Ok(Event::ButtonPressed(ButtonKind::Strong)) => {
                if *level < 9 {
                    *level += 1;
                    State::LevelUp
                } else {
                    State::PowerOn
                }
            }
            Ok(Event::ProximityChanged(p)) => {
                if p > PAN_OFF_TH {
                    State::Standby
                } else {
                    State::PanShake
                }
            }
            _ => State::Nikomi,
        },
        State::LevelDown => State::PowerOn,
        State::LevelUp => State::PowerOn,
    }
}

fn nikomi_animation(
    time_stamp: u16,
    pwm0: &mut PioPwm<'static, PIO0, 0>,
    pwm1: &mut PioPwm<'static, PIO0, 1>,
) {
    let fts = time_stamp as f32 / 75.0;
    let freq0: f32 = (fts).sin() + 0.5;
    let freq1: f32 = (fts * 2.0).sin() + 0.5;
    let freq2: f32 = (fts * 3.8).sin() + 0.5;
    let combined: f32 = freq0 * freq1 * freq2;
    let duty_percent = (combined.clamp(0.1, 1.0) * 100.0) as u8;
    let duty = core_time::Duration::from_micros(MAX_DUTY_MICRO * duty_percent as u64 / 100);
    pwm0.write(duty);
    pwm1.write(duty);
}

async fn power_off_animation(
    level: u8,
    pwm0: &mut PioPwm<'static, PIO0, 0>,
    pwm1: &mut PioPwm<'static, PIO0, 1>,
) {
    let start_duty_percent = (level + 1) * 10;
    for duty_percent in (0..=start_duty_percent).rev() {
        let duty = core_time::Duration::from_micros(MAX_DUTY_MICRO * duty_percent as u64 / 100);
        pwm0.write(duty);
        pwm1.write(duty);
        em_time::Timer::after(em_time::Duration::from_millis(10)).await;
    }
    pwm0.write(core_time::Duration::from_micros(0));
    pwm1.write(core_time::Duration::from_micros(0));
}

async fn standby_animation(
    level: u8,
    pwm0: &mut PioPwm<'static, PIO0, 0>,
    pwm1: &mut PioPwm<'static, PIO0, 1>,
) {
    let target_duty_percent = (level + 1) * 10;
    for duty_percent in 0..=target_duty_percent {
        let duty = core_time::Duration::from_micros(MAX_DUTY_MICRO * duty_percent as u64 / 100);
        pwm0.write(duty);
        pwm1.write(duty);
        em_time::Timer::after(em_time::Duration::from_millis(10)).await;
    }
    for duty_percent in (0..=target_duty_percent).rev() {
        let duty = core_time::Duration::from_micros(MAX_DUTY_MICRO * duty_percent as u64 / 100);
        pwm0.write(duty);
        pwm1.write(duty);
        em_time::Timer::after(em_time::Duration::from_millis(10)).await;
    }
}

async fn power_on_animation(
    level: u8,
    pwm0: &mut PioPwm<'static, PIO0, 0>,
    pwm1: &mut PioPwm<'static, PIO0, 1>,
) {
    let target_duty_percent = (level + 1) * 10;
    pwm0.write(core_time::Duration::from_micros(
        MAX_DUTY_MICRO * target_duty_percent as u64 / 100,
    ));
    pwm1.write(core_time::Duration::from_micros(
        MAX_DUTY_MICRO * target_duty_percent as u64 / 100,
    ));
}

async fn pan_shake_animation(
    level: u8,
    pwm0: &mut PioPwm<'static, PIO0, 0>,
    pwm1: &mut PioPwm<'static, PIO0, 1>,
) {
    let target_duty_percent = (level + 1) * 10;
    for _ in 0..=1 {
        for duty_percent in 0..=target_duty_percent {
            let duty = core_time::Duration::from_micros(MAX_DUTY_MICRO * duty_percent as u64 / 100);
            pwm0.write(duty);
            pwm1.write(duty);
            em_time::Timer::after(em_time::Duration::from_millis(5)).await;
        }
    }
}

async fn level_change_animation(
    level: u8,
    pwm0: &mut PioPwm<'static, PIO0, 0>,
    pwm1: &mut PioPwm<'static, PIO0, 1>,
) {
    let target_duty_percent = (level + 1) * 10;
    pwm1.write(core_time::Duration::from_micros(
        MAX_DUTY_MICRO * target_duty_percent as u64 / 100,
    ));
    for duty_percent in (0..=100).skip(50) {
        let duty = core_time::Duration::from_micros(MAX_DUTY_MICRO * duty_percent as u64 / 100);
        pwm0.write(duty);
        em_time::Timer::after(em_time::Duration::from_millis(5)).await;
    }
    for duty_percent in (target_duty_percent..=100).rev().skip(50) {
        let duty = core_time::Duration::from_micros(MAX_DUTY_MICRO * duty_percent as u64 / 100);
        pwm0.write(duty);
        em_time::Timer::after(em_time::Duration::from_millis(5)).await;
    }
}
