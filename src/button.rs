//! Button handling task
//!

use embassy_rp::gpio::Input;

use crate::{ButtonKind, CHANNEL, Event};

#[embassy_executor::task]
pub async fn button_nikomi_handler_task(mut button: Input<'static>) {
    loop {
        button.wait_for_falling_edge().await;
        CHANNEL.send(Event::ButtonPressed(ButtonKind::Nikomi)).await;
    }
}
#[embassy_executor::task]
pub async fn button_weak_handler_task(mut button: Input<'static>) {
    loop {
        button.wait_for_falling_edge().await;
        CHANNEL.send(Event::ButtonPressed(ButtonKind::Weak)).await;
    }
}
#[embassy_executor::task]
pub async fn button_strong_handler_task(mut button: Input<'static>) {
    loop {
        button.wait_for_falling_edge().await;
        CHANNEL.send(Event::ButtonPressed(ButtonKind::Strong)).await;
    }
}
#[embassy_executor::task]
pub async fn button_power_handler_task(mut button: Input<'static>) {
    loop {
        button.wait_for_falling_edge().await;
        CHANNEL.send(Event::ButtonPressed(ButtonKind::Power)).await;
    }
}
