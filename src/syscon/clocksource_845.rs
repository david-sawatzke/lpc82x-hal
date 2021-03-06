use crate::pac;
use crate::{
    pac::syscon::fclksel::SEL_A,
    syscon::{self, frg, PeripheralClock, IOSC},
};

use core::marker::PhantomData;

/// Internal trait used for defining the fclksel index for a peripheral
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait PeripheralClockSelector {
    /// The index
    const REGISTER_NUM: usize;
}

macro_rules! periph_clock_selector {
    ($peripheral:ident, $num:expr) => {
        impl PeripheralClockSelector for pac::$peripheral {
            const REGISTER_NUM: usize = $num;
        }
    };
}

periph_clock_selector!(USART0, 0);
periph_clock_selector!(USART1, 1);
periph_clock_selector!(USART2, 2);
periph_clock_selector!(USART3, 3);
periph_clock_selector!(USART4, 4);
periph_clock_selector!(I2C0, 5);
periph_clock_selector!(I2C1, 6);
periph_clock_selector!(I2C2, 7);
periph_clock_selector!(I2C3, 8);

/// Internal trait used for defining valid peripheal clock sources
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait PeripheralClockSource {
    /// The variant
    const CLOCK: SEL_A;
}

impl PeripheralClockSource for frg::FRG<frg::FRG0> {
    const CLOCK: SEL_A = SEL_A::FRG0CLK;
}

impl PeripheralClockSource for frg::FRG<frg::FRG1> {
    const CLOCK: SEL_A = SEL_A::FRG1CLK;
}

impl PeripheralClockSource for IOSC {
    const CLOCK: SEL_A = SEL_A::FRO;
}

/// Defines the clock configuration for a usart
pub struct UsartClock<PeriphClock> {
    pub(crate) psc: u16,
    pub(crate) osrval: u8,
    _periphclock: PhantomData<PeriphClock>,
}

impl<PERIPH: crate::usart::Instance, CLOCK: PeripheralClockSource>
    UsartClock<(PERIPH, CLOCK)>
{
    /// Create the clock config for the uart
    ///
    /// `osrval` has to be between 5-16
    pub fn new(_: &CLOCK, psc: u16, osrval: u8) -> Self {
        let osrval = osrval - 1;
        assert!(osrval > 3 && osrval < 0x10);

        Self {
            psc,
            osrval,
            _periphclock: PhantomData,
        }
    }
}

impl<PERIPH: crate::usart::Instance + PeripheralClockSelector>
    UsartClock<(PERIPH, IOSC)>
{
    /// Create a new configuration with a specified baudrate
    ///
    /// Assumes the internal oscillator runs at 12 MHz
    pub fn new_with_baudrate(baudrate: u32) -> Self {
        // We want something with 5% tolerance
        let calc = baudrate * 20;
        let mut osrval = 5;
        for i in (5..=16).rev() {
            if calc * (i as u32) < 12_000_000 {
                osrval = i;
            }
        }
        let psc = (12_000_000 / (baudrate * osrval as u32) - 1) as u16;
        let osrval = osrval - 1;
        Self {
            psc,
            osrval,
            _periphclock: PhantomData,
        }
    }
}

impl<PERIPH: PeripheralClockSelector, CLOCK: PeripheralClockSource>
    PeripheralClock<PERIPH> for UsartClock<(PERIPH, CLOCK)>
{
    fn select_clock(&self, syscon: &mut syscon::Handle) {
        syscon.fclksel[PERIPH::REGISTER_NUM]
            .write(|w| w.sel().variant(CLOCK::CLOCK));
    }
}

/// A struct containing the clock configuration for a peripheral
pub struct I2cClock<PeriphClock> {
    pub(crate) divval: u16,
    pub(crate) mstsclhigh: u8,
    pub(crate) mstscllow: u8,
    _periphclock: PhantomData<PeriphClock>,
}

impl<PERIPH: PeripheralClockSelector, CLOCK: PeripheralClockSource>
    I2cClock<(PERIPH, CLOCK)>
{
    /// Create the clock config for the i2c peripheral
    ///
    /// mstclhigh & mstcllow have to be between 2-9
    pub fn new(_: &CLOCK, divval: u16, mstsclhigh: u8, mstscllow: u8) -> Self {
        assert!(mstsclhigh > 1 && mstsclhigh < 10);
        assert!(mstscllow > 1 && mstscllow < 10);
        Self {
            divval,
            mstsclhigh: mstsclhigh - 2,
            mstscllow: mstscllow - 2,
            _periphclock: PhantomData,
        }
    }
}

impl<PERIPH: PeripheralClockSelector> I2cClock<(PERIPH, IOSC)> {
    /// Create a new i2c clock config for 400 kHz
    ///
    /// Assumes the internal oscillator runs at 12 MHz
    pub fn new_400khz() -> Self {
        Self {
            divval: 5,
            mstsclhigh: 0,
            mstscllow: 1,
            _periphclock: PhantomData,
        }
    }
}

impl<PERIPH: PeripheralClockSelector, CLOCK: PeripheralClockSource>
    PeripheralClock<PERIPH> for I2cClock<(PERIPH, CLOCK)>
{
    fn select_clock(&self, syscon: &mut syscon::Handle) {
        syscon.fclksel[PERIPH::REGISTER_NUM]
            .write(|w| w.sel().variant(CLOCK::CLOCK));
    }
}
