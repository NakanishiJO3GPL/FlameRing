//! Animation engine
//!

use core::time as core_time;
//use defmt::info;
use embassy_rp::{peripherals::PIO0, pio_programs::pwm::PioPwm};
use embassy_time as em_time;
use micromath::F32Ext;

const NIKOMI_ANIMATION_STEPS: f32 = 75.0;
const MAX_DUTY_MICRO: u64 = 2_500;

pub struct AnimationEngine {
    pwm0: PioPwm<'static, PIO0, 0>,
    pwm1: PioPwm<'static, PIO0, 1>,
    time_stamp: u16,
}

impl AnimationEngine {
    pub fn new(pwm0: PioPwm<'static, PIO0, 0>, pwm1: PioPwm<'static, PIO0, 1>) -> Self {
        Self {
            pwm0,
            pwm1,
            time_stamp: 0,
        }
    }

    pub fn nikomi(&mut self) {
        let fts = self.time_stamp as f32 / NIKOMI_ANIMATION_STEPS;
        let freq0 = fts.sin() + 0.5;
        let freq1 = (fts * 2.0).sin() + 0.5;
        let freq2 = (fts * 3.8).sin() + 0.5;
        let combined = freq0 * freq1 * freq2;
        let duty_percent = (combined.clamp(0.1, 1.0) * 100.0) as u8;
        let duty = core_time::Duration::from_micros(MAX_DUTY_MICRO * duty_percent as u64 / 100);
        self.pwm0.write(duty);
        self.pwm1.write(duty);
        self.time_stamp = self.time_stamp.wrapping_add(1);
    }

    pub async fn power_off(&mut self, level: &u8) {
        let start_duty_percent = (level + 1) * 10;
        for duty_percent in (0..=start_duty_percent).rev() {
            let duty = core_time::Duration::from_micros(MAX_DUTY_MICRO * duty_percent as u64 / 100);
            self.pwm0.write(duty);
            self.pwm1.write(duty);
            em_time::Timer::after(em_time::Duration::from_millis(10)).await;
        }
        self.pwm0.write(core_time::Duration::from_micros(0));
        self.pwm1.write(core_time::Duration::from_micros(0));
    }

    pub async fn standby(&mut self, level: &u8) {
        let target_duty_percent = (level + 1) * 10;
        for duty_percent in 0..=target_duty_percent {
            let duty = core_time::Duration::from_micros(MAX_DUTY_MICRO * duty_percent as u64 / 100);
            self.pwm0.write(duty);
            self.pwm1.write(duty);
            em_time::Timer::after(em_time::Duration::from_millis(10)).await;
        }
        for duty_percent in (0..=target_duty_percent).rev() {
            let duty = core_time::Duration::from_micros(MAX_DUTY_MICRO * duty_percent as u64 / 100);
            self.pwm0.write(duty);
            self.pwm1.write(duty);
            em_time::Timer::after(em_time::Duration::from_millis(10)).await;
        }
        // Wait for welcome animation finish
        em_time::Timer::after(em_time::Duration::from_millis(500)).await
    }

    pub async fn power_on(&mut self, level: &u8) {
        let target_duty_percent = (level + 1) * 10;
        let duty =
            core_time::Duration::from_micros(MAX_DUTY_MICRO * target_duty_percent as u64 / 100);
        self.pwm0.write(duty);
        self.pwm1.write(duty);
    }

    pub async fn pan_shake(&mut self, level: &u8) {
        let target_duty_percent = (level + 1) * 10;
        for _ in 0..=1 {
            for duty_percent in 0..=target_duty_percent {
                let duty =
                    core_time::Duration::from_micros(MAX_DUTY_MICRO * duty_percent as u64 / 100);
                self.pwm0.write(duty);
                self.pwm1.write(duty);
                em_time::Timer::after(em_time::Duration::from_millis(5)).await;
            }
        }
        // Wait for animation
        em_time::Timer::after(em_time::Duration::from_millis(500)).await
    }

    pub async fn level_change(&mut self, level: &u8) {
        let target_duty_percent = (level + 1) * 10;
        self.pwm1.write(core_time::Duration::from_micros(
            MAX_DUTY_MICRO * target_duty_percent as u64 / 100,
        ));
        for duty_percent in (0..=100).skip(50) {
            let duty = core_time::Duration::from_micros(MAX_DUTY_MICRO * duty_percent as u64 / 100);
            self.pwm0.write(duty);
            em_time::Timer::after(em_time::Duration::from_millis(5)).await;
        }
        for duty_percent in (target_duty_percent..=100).rev().skip(50) {
            let duty = core_time::Duration::from_micros(MAX_DUTY_MICRO * duty_percent as u64 / 100);
            self.pwm0.write(duty);
            em_time::Timer::after(em_time::Duration::from_millis(5)).await;
        }
    }
}
