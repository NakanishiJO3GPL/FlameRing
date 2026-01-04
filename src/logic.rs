//! Animation logic
//!

use core::time as core_time;
use defmt::info;
use embassy_rp::{peripherals::PIO0, pio_programs::pwm::PioPwm};
use micromath::F32Ext;
