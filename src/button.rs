//! Button handling task
//!

use crate::{ButtonKind, CHANNEL, Event};
use embassy_rp::gpio::Input;
use embassy_time::{Duration, Timer};

const DEBOUNCE_DELAY_MS: u64 = 30;

#[embassy_executor::task]
pub async fn button_nikomi_handler_task(mut button: Input<'static>) {
    loop {
        button.wait_for_falling_edge().await;
        Timer::after(Duration::from_millis(DEBOUNCE_DELAY_MS)).await; // Debounce delay
        if button.is_low() {
            CHANNEL.send(Event::ButtonPressed(ButtonKind::Nikomi)).await;
        }
    }
}
#[embassy_executor::task]
pub async fn button_weak_handler_task(mut button: Input<'static>) {
    loop {
        button.wait_for_falling_edge().await;
        Timer::after(Duration::from_millis(DEBOUNCE_DELAY_MS)).await; // Debounce delay
        if button.is_low() {
            CHANNEL.send(Event::ButtonPressed(ButtonKind::Weak)).await;
        }
    }
}
#[embassy_executor::task]
pub async fn button_strong_handler_task(mut button: Input<'static>) {
    loop {
        button.wait_for_falling_edge().await;
        Timer::after(Duration::from_millis(DEBOUNCE_DELAY_MS)).await; // Debounce delay
        if button.is_low() {
            CHANNEL.send(Event::ButtonPressed(ButtonKind::Strong)).await;
        }
    }
}
#[embassy_executor::task]
pub async fn button_power_handler_task(mut button: Input<'static>) {
    loop {
        button.wait_for_falling_edge().await;
        Timer::after(Duration::from_millis(DEBOUNCE_DELAY_MS)).await; // Debounce delay
        if button.is_low() {
            CHANNEL.send(Event::ButtonPressed(ButtonKind::Power)).await;
        }
    }
}
