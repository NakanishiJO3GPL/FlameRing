//! Animation task
//!

use defmt::info;
use embassy_rp::{peripherals::PIO0, pio_programs::pwm::PioPwm};
use embassy_time as em_time;

use crate::{ButtonKind, CHANNEL, Event, animation::AnimationEngine};

#[derive(Copy, Clone, PartialEq, Eq, Debug, defmt::Format)]
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

const SAMPLING_INTERVAL_MS: u64 = 10;

#[embassy_executor::task]
pub async fn animation_state_task(pwm0: PioPwm<'static, PIO0, 0>, pwm1: PioPwm<'static, PIO0, 1>) {
    let mut state = State::default();
    let mut prev_state = state;
    let mut animation_engine = AnimationEngine::new(pwm0, pwm1);
    let mut level: u8 = 5;

    loop {
        state = update_state(state, &mut level);

        // Update LED
        if state == State::Nikomi {
            animation_engine.nikomi();
        } else if prev_state != state {
            prev_state = state;
            match state {
                State::PowerOff => {
                    info!("LED Off");
                    animation_engine.power_off(&level).await;
                }
                State::Standby => {
                    info!("LED Standby");
                    animation_engine.standby(&level).await;
                }
                State::PowerOn => {
                    info!("LED PowerOn");
                    animation_engine.power_on(&level).await;
                }
                State::PanShake => {
                    info!("LED PanShake");
                    animation_engine.pan_shake(&level).await;
                }
                State::LevelDown => {
                    info!("LED Level Down to {}", level);
                    animation_engine.level_change(&level).await;
                }
                State::LevelUp => {
                    info!("LED Level Up to {}", level);
                    animation_engine.level_change(&level).await;
                }
                _ => {
                    // Do nothing
                }
            }
        }

        em_time::Timer::after(em_time::Duration::from_millis(SAMPLING_INTERVAL_MS)).await;
    }
}

fn update_state(state: State, level: &mut u8) -> State {
    info!("Current State: {:?}", state);
    let event = CHANNEL.try_receive();
    match (state, event) {
        (State::PowerOff, Ok(Event::ButtonPressed(ButtonKind::Power))) => State::Standby,
        (State::PowerOff, _) => State::PowerOff,

        (State::Standby, Ok(Event::ButtonPressed(ButtonKind::Power))) => State::PowerOff,
        (State::Standby, Ok(Event::ProximityPanOn)) => State::PowerOn,
        (State::Standby, Ok(Event::ProximityPanOff)) => State::Standby,
        (State::Standby, _) => State::Standby,

        (State::PowerOn, Ok(Event::ButtonPressed(ButtonKind::Power))) => State::PowerOff,
        (State::PowerOn, Ok(Event::ButtonPressed(ButtonKind::Weak))) => level_down(level),
        (State::PowerOn, Ok(Event::ButtonPressed(ButtonKind::Strong))) => level_up(level),
        (State::PowerOn, Ok(Event::ButtonPressed(ButtonKind::Nikomi))) => State::Nikomi,
        (State::PowerOn, Ok(Event::ProximityChanged(_))) => State::PanShake,
        (State::PowerOn, Ok(Event::ProximityPanOn)) => State::PowerOn,
        (State::PowerOn, Ok(Event::ProximityPanOff)) => State::Standby,
        (State::PowerOn, _) => State::PowerOn,

        (State::PanShake, Ok(Event::ButtonPressed(ButtonKind::Power))) => State::PowerOff,
        (State::PanShake, Ok(Event::ProximityChanged(_))) => State::PanShake,
        (State::PanShake, Ok(Event::ProximityPanOff)) => State::Standby,
        (State::PanShake, _) => State::PowerOn,

        (State::Nikomi, Ok(Event::ButtonPressed(ButtonKind::Power))) => State::PowerOff,
        (State::Nikomi, Ok(Event::ButtonPressed(ButtonKind::Weak))) => level_down(level),
        (State::Nikomi, Ok(Event::ButtonPressed(ButtonKind::Strong))) => level_up(level),
        (State::Nikomi, Ok(Event::ProximityPanOn)) => State::Nikomi,
        (State::Nikomi, Ok(Event::ProximityPanOff)) => State::Standby,
        (State::Nikomi, _) => State::Nikomi,

        (State::LevelDown, _) => State::PowerOn,
        (State::LevelUp, _) => State::PowerOn,
    }
}

fn level_up(level: &mut u8) -> State {
    if *level < 9 {
        *level += 1;
        State::LevelUp
    } else {
        State::PowerOn
    }
}

fn level_down(level: &mut u8) -> State {
    if *level > 0 {
        *level -= 1;
        State::LevelDown
    } else {
        State::PowerOn
    }
}
