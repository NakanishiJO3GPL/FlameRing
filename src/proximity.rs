//! Proximity sensor task
//!

use embassy_rp::{
    i2c::{self, I2c},
    peripherals::I2C0,
};
use embassy_time as em_time;

use crate::{CHANNEL, Event};

const PROXIMITY_SENSOR_ADDR: u8 = 0x38;
const PROXIMITY_SENSOR_REG: u8 = 0x44;
const PROXIMITY_SENSOR_INIT_SEQ: &[(u8, u8)] = &[
    (0x40, 0xc0), // bp7:SW reset, bp6:INT reset
    (0x40, 0x00), // bp7:SW reset, bp6:INT reset
    (0x43, 0x01), // bp5-4:PS_GAIN, bp3-0:PERSISTENCE
    (0x42, 0x02), // bp5-4:ALS DATA0 gain, bp3-2:ALS DATA1 gain, bp1-0:LED current
    (0x41, 0xc6), // Measurement start
];
const PROXIMITY_SENSOR_CHANGE_TH: i32 = 10;

#[embassy_executor::task]
pub async fn proximity_sensor_task(mut i2c: I2c<'static, I2C0, i2c::Async>) {
    // Initialize proximity sensor
    for &(reg, val) in PROXIMITY_SENSOR_INIT_SEQ {
        let addr: u16 = PROXIMITY_SENSOR_ADDR as u16;
        let buf = (((reg as u16) << 8) | (val as u16)).to_be_bytes();
        let _ = i2c.write_async(addr, buf).await;
        em_time::Timer::after(em_time::Duration::from_millis(10)).await;
    }

    // Main loop
    let mut prev_proximity: u16 = 0;
    loop {
        let addr: u16 = PROXIMITY_SENSOR_ADDR as u16;
        let reg = PROXIMITY_SENSOR_REG.to_be_bytes();
        let mut buf = [0u8; 2];
        match i2c.write_read_async(addr, reg, &mut buf).await {
            Ok(()) => {
                let proximity = 4095 - u16::from_le_bytes(buf);
                if (proximity as i32 - prev_proximity as i32).abs() > PROXIMITY_SENSOR_CHANGE_TH {
                    CHANNEL.send(Event::ProximityChanged(proximity)).await;
                    prev_proximity = proximity;
                }
                CHANNEL.send(Event::ProximityCurrent(proximity)).await;
            }
            Err(_) => {
                // Handle read error (e.g., log it)
            }
        }

        em_time::Timer::after(em_time::Duration::from_millis(50)).await;
    }
}
