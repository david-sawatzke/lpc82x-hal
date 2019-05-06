use core::marker::PhantomData;

use crate::raw_compat::syscon::{
    pdruncfg, presetctrl, starterp1, sysahbclkctrl, PDRUNCFG, PRESETCTRL, STARTERP1, SYSAHBCLKCTRL,
};

use crate::reg;
// TODO Remove when FRO is implemented for lpc845
#[allow(unused_imports)]
use crate::{clock, init_state, raw, raw_compat, reg_proxy::RegProxy};

pub struct CommonParts {
    /// The handle to the SYSCON peripheral
    pub handle: Handle,

    /// Brown-out detection
    pub bod: BOD,

    /// Flash memory
    pub flash: FLASH,

    /// Micro Trace Buffer
    pub mtb: MTB,

    /// Random access memory
    pub ram0_1: RAM0_1,

    /// Read-only memory
    pub rom: ROM,

    /// System oscillator
    pub sysosc: SYSOSC,

    /// PLL
    pub syspll: SYSPLL,
}

impl CommonParts {
    pub unsafe fn new() -> CommonParts {
        CommonParts {
            handle: Handle {
                pdruncfg: RegProxy::new(),
                presetctrl: RegProxy::new(),
                starterp1: RegProxy::new(),
                sysahbclkctrl: RegProxy::new(),
            },

            bod: BOD(PhantomData),
            flash: FLASH(PhantomData),
            mtb: MTB(PhantomData),
            ram0_1: RAM0_1(PhantomData),
            rom: ROM(PhantomData),
            sysosc: SYSOSC(PhantomData),
            syspll: SYSPLL(PhantomData),
        }
    }
}

/// Handle to the SYSCON peripheral
///
/// This handle to the SYSCON peripheral provides access to the main part of the
/// SYSCON API. It is also required by other parts of the HAL API to synchronize
/// access the the underlying registers, wherever this is required.
///
/// Please refer to the [module documentation] for more information about the
/// PMU.
///
/// [module documentation]: index.html
pub struct Handle {
    pdruncfg: RegProxy<PDRUNCFG>,
    presetctrl: RegProxy<PRESETCTRL>,
    starterp1: RegProxy<STARTERP1>,
    sysahbclkctrl: RegProxy<SYSAHBCLKCTRL>,
}

impl Handle {
    /// Enable peripheral clock
    ///
    /// Enables the clock for a peripheral or other hardware component. HAL
    /// users usually won't have to call this method directly, as other
    /// peripheral APIs will do this for them.
    pub fn enable_clock<P: ClockControl>(&mut self, peripheral: &P) {
        self.sysahbclkctrl.modify(|_, w| peripheral.enable_clock(w));
    }

    /// Disable peripheral clock
    pub fn disable_clock<P: ClockControl>(&mut self, peripheral: &P) {
        self.sysahbclkctrl
            .modify(|_, w| peripheral.disable_clock(w));
    }

    /// Assert peripheral reset
    pub fn assert_reset<P: ResetControl>(&mut self, peripheral: &P) {
        self.presetctrl.modify(|_, w| peripheral.assert_reset(w));
    }

    /// Clear peripheral reset
    ///
    /// Clears the reset for a peripheral or other hardware component. HAL users
    /// usually won't have to call this method directly, as other peripheral
    /// APIs will do this for them.
    pub fn clear_reset<P: ResetControl>(&mut self, peripheral: &P) {
        self.presetctrl.modify(|_, w| peripheral.clear_reset(w));
    }

    /// Provide power to an analog block
    ///
    /// HAL users usually won't have to call this method themselves, as other
    /// peripheral APIs will do this for them.
    pub fn power_up<P: AnalogBlock>(&mut self, peripheral: &P) {
        self.pdruncfg.modify(|_, w| peripheral.power_up(w));
    }

    /// Remove power from an analog block
    pub fn power_down<P: AnalogBlock>(&mut self, peripheral: &P) {
        self.pdruncfg.modify(|_, w| peripheral.power_down(w));
    }

    /// Enable interrupt wake-up from deep-sleep and power-down modes
    ///
    /// To use an interrupt for waking up the system from the deep-sleep and
    /// power-down modes, it needs to be enabled using this method, in addition
    /// to being enabled in the NVIC.
    ///
    /// This method is not required when using the regular sleep mode.
    pub fn enable_interrupt_wakeup<I>(&mut self)
    where
        I: WakeUpInterrupt,
    {
        self.starterp1.modify(|_, w| I::enable(w));
    }

    /// Disable interrupt wake-up from deep-sleep and power-down modes
    pub fn disable_interrupt_wakeup<I>(&mut self)
    where
        I: WakeUpInterrupt,
    {
        self.starterp1.modify(|_, w| I::disable(w));
    }
}

/// Brown-out detection
///
/// Can be used to control brown-out detection using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct BOD(PhantomData<*const ()>);

/// Flash memory
///
/// Can be used to control flash memory using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct FLASH(PhantomData<*const ()>);

/// Micro Trace Buffer
///
/// Can be used to control the Micro Trace Buffer using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct MTB(PhantomData<*const ()>);

/// Random access memory
///
/// Can be used to control the RAM using various methods on [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
#[allow(non_camel_case_types)]
pub struct RAM0_1(PhantomData<*const ()>);

/// Read-only memory
///
/// Can be used to control the ROM using various methods on [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct ROM(PhantomData<*const ()>);

/// System oscillator
///
/// Can be used to control the system oscillator using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct SYSOSC(PhantomData<*const ()>);

/// PLL
///
/// Can be used to control the PLL using various methods on [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct SYSPLL(PhantomData<*const ()>);

/// Internal trait for controlling peripheral clocks
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any changes to this trait won't
/// be considered breaking changes.
///
/// Please refer to [`syscon::Handle::enable_clock`] and
/// [`syscon::Handle::disable_clock`] for the public API that uses this trait.
///
/// [`syscon::Handle::enable_clock`]: struct.Handle.html#method.enable_clock
/// [`syscon::Handle::disable_clock`]: struct.Handle.html#method.disable_clock
pub trait ClockControl {
    /// Internal method to enable a peripheral clock
    fn enable_clock<'w>(&self, w: &'w mut sysahbclkctrl::W) -> &'w mut sysahbclkctrl::W;

    /// Internal method to disable a peripheral clock
    fn disable_clock<'w>(&self, w: &'w mut sysahbclkctrl::W) -> &'w mut sysahbclkctrl::W;
}

macro_rules! impl_clock_control {
    ($clock_control:ty, $clock:ident) => {
        impl ClockControl for $clock_control {
            fn enable_clock<'w>(&self, w: &'w mut sysahbclkctrl::W) -> &'w mut sysahbclkctrl::W {
                w.$clock().enable()
            }

            fn disable_clock<'w>(&self, w: &'w mut sysahbclkctrl::W) -> &'w mut sysahbclkctrl::W {
                w.$clock().disable()
            }
        }
    };
}

impl_clock_control!(ROM, rom);
impl_clock_control!(RAM0_1, ram0_1);
#[cfg(feature = "82x")]
impl_clock_control!(raw_compat::FLASH_CTRL, flashreg);
#[cfg(feature = "845")]
impl_clock_control!(raw_compat::FLASH_CTRL, flash);
impl_clock_control!(FLASH, flash);
impl_clock_control!(raw::I2C0, i2c0);
#[cfg(feature = "82x")]
impl_clock_control!(raw_compat::GPIO, gpio);
impl_clock_control!(raw_compat::SWM0, swm);
impl_clock_control!(raw_compat::SCT0, sct);
impl_clock_control!(raw::WKT, wkt);
impl_clock_control!(raw_compat::MRT0, mrt);
impl_clock_control!(raw::SPI0, spi0);
impl_clock_control!(raw::SPI1, spi1);
impl_clock_control!(raw::CRC, crc);
impl_clock_control!(raw::USART0, uart0);
impl_clock_control!(raw::USART1, uart1);
impl_clock_control!(raw::USART2, uart2);
impl_clock_control!(raw::WWDT, wwdt);
impl_clock_control!(raw::IOCON, iocon);
impl_clock_control!(raw_compat::ACOMP, acmp);
impl_clock_control!(raw::I2C1, i2c1);
impl_clock_control!(raw::I2C2, i2c2);
impl_clock_control!(raw::I2C3, i2c3);
impl_clock_control!(raw_compat::ADC0, adc);
impl_clock_control!(MTB, mtb);
impl_clock_control!(raw_compat::DMA0, dma);

#[cfg(feature = "845")]
impl ClockControl for raw_compat::GPIO {
    fn enable_clock<'w>(&self, w: &'w mut sysahbclkctrl::W) -> &'w mut sysahbclkctrl::W {
        w.gpio0().enable().gpio1().enable()
    }

    fn disable_clock<'w>(&self, w: &'w mut sysahbclkctrl::W) -> &'w mut sysahbclkctrl::W {
        w.gpio0().disable().gpio1().disable()
    }
}

/// Internal trait for controlling peripheral reset
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any incompatible changes to this
/// trait won't be considered breaking changes.
///
/// Please refer to [`syscon::Handle::assert_reset`] and
/// [`syscon::Handle::clear_reset`] for the public API that uses this trait.
///
/// [`syscon::Handle::assert_reset`]: struct.Handle.html#method.assert_reset
/// [`syscon::Handle::clear_reset`]: struct.Handle.html#method.clear_reset
pub trait ResetControl {
    /// Internal method to assert peripheral reset
    fn assert_reset<'w>(&self, w: &'w mut presetctrl::W) -> &'w mut presetctrl::W;

    /// Internal method to clear peripheral reset
    fn clear_reset<'w>(&self, w: &'w mut presetctrl::W) -> &'w mut presetctrl::W;
}

#[macro_export]
macro_rules! impl_reset_control {
    ($reset_control:ty, $field:ident) => {
        impl<'a> ResetControl for $reset_control {
            fn assert_reset<'w>(&self, w: &'w mut presetctrl::W) -> &'w mut presetctrl::W {
                w.$field().clear_bit()
            }

            fn clear_reset<'w>(&self, w: &'w mut presetctrl::W) -> &'w mut presetctrl::W {
                w.$field().set_bit()
            }
        }
    };
}

impl_reset_control!(raw::SPI0, spi0_rst_n);
impl_reset_control!(raw::SPI1, spi1_rst_n);
impl_reset_control!(raw::USART0, uart0_rst_n);
impl_reset_control!(raw::USART1, uart1_rst_n);
impl_reset_control!(raw::USART2, uart2_rst_n);
impl_reset_control!(raw::I2C0, i2c0_rst_n);
impl_reset_control!(raw_compat::MRT0, mrt_rst_n);
impl_reset_control!(raw_compat::SCT0, sct_rst_n);
impl_reset_control!(raw::WKT, wkt_rst_n);
impl_reset_control!(raw_compat::FLASH_CTRL, flash_rst_n);
impl_reset_control!(raw_compat::ACOMP, acmp_rst_n);
impl_reset_control!(raw::I2C1, i2c1_rst_n);
impl_reset_control!(raw::I2C2, i2c2_rst_n);
impl_reset_control!(raw::I2C3, i2c3_rst_n);
impl_reset_control!(raw_compat::ADC0, adc_rst_n);
impl_reset_control!(raw_compat::DMA0, dma_rst_n);
#[cfg(feature = "82x")]
impl_reset_control!(raw_compat::GPIO, gpio_rst_n);

#[cfg(feature = "845")]
impl<'a> ResetControl for raw_compat::GPIO {
    fn assert_reset<'w>(&self, w: &'w mut presetctrl::W) -> &'w mut presetctrl::W {
        w.gpio0_rst_n().clear_bit().gpio1_rst_n().clear_bit()
    }

    fn clear_reset<'w>(&self, w: &'w mut presetctrl::W) -> &'w mut presetctrl::W {
        w.gpio0_rst_n().set_bit().gpio1_rst_n().set_bit()
    }
}

/// Internal trait for powering analog blocks
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any changes to this trait won't
/// be considered breaking changes.
///
/// Please refer to [`syscon::Handle::power_up`] and
/// [`syscon::Handle::power_down`] for the public API that uses this trait.
///
/// [`syscon::Handle::power_up`]: struct.Handle.html#method.power_up
/// [`syscon::Handle::power_down`]: struct.Handle.html#method.power_down
pub trait AnalogBlock {
    /// Internal method to power up an analog block
    fn power_up<'w>(&self, w: &'w mut pdruncfg::W) -> &'w mut pdruncfg::W;

    /// Internal method to power down an analog block
    fn power_down<'w>(&self, w: &'w mut pdruncfg::W) -> &'w mut pdruncfg::W;
}

#[macro_export]
macro_rules! impl_analog_block {
    ($analog_block:ty, $field:ident) => {
        impl<'a> AnalogBlock for $analog_block {
            fn power_up<'w>(&self, w: &'w mut pdruncfg::W) -> &'w mut pdruncfg::W {
                w.$field().clear_bit()
            }

            fn power_down<'w>(&self, w: &'w mut pdruncfg::W) -> &'w mut pdruncfg::W {
                w.$field().set_bit()
            }
        }
    };
}

impl_analog_block!(FLASH, flash_pd);
impl_analog_block!(BOD, bod_pd);
impl_analog_block!(raw_compat::ADC0, adc_pd);
impl_analog_block!(SYSOSC, sysosc_pd);
impl_analog_block!(raw::WWDT, wdtosc_pd);
impl_analog_block!(SYSPLL, syspll_pd);
impl_analog_block!(raw_compat::ACOMP, acmp);

/// Internal trait used to configure interrupt wake-up
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any changes to this trait won't
/// be considered breaking changes.
///
/// Please refer to [`syscon::Handle::enable_interrupt_wakeup`] and
/// [`syscon::Handle::disable_interrupt_wakeup`] for the public API that uses
/// this trait.
///
/// [`syscon::Handle::enable_interrupt_wakeup`]: struct.Handle.html#method.enable_interrupt_wakeup
/// [`syscon::Handle::disable_interrupt_wakeup`]: struct.Handle.html#method.disable_interrupt_wakeup
pub trait WakeUpInterrupt {
    /// Internal method to configure interrupt wakeup behavior
    fn enable(w: &mut starterp1::W) -> &mut starterp1::W;

    /// Internal method to configure interrupt wakeup behavior
    fn disable(w: &mut starterp1::W) -> &mut starterp1::W;
}

macro_rules! wakeup_interrupt {
    ($name:ident, $field:ident) => {
        /// Can be used to enable/disable interrupt wake-up behavior
        ///
        /// See [`syscon::Handle::enable_interrupt_wakeup`] and
        /// [`syscon::Handle::disable_interrupt_wakeup`].
        ///
        /// [`syscon::Handle::enable_interrupt_wakeup`]: struct.Handle.html#method.enable_interrupt_wakeup
        /// [`syscon::Handle::disable_interrupt_wakeup`]: struct.Handle.html#method.disable_interrupt_wakeup
        pub struct $name;

        impl WakeUpInterrupt for $name {
            fn enable(w: &mut starterp1::W) -> &mut starterp1::W {
                w.$field().enabled()
            }

            fn disable(w: &mut starterp1::W) -> &mut starterp1::W {
                w.$field().disabled()
            }
        }
    };
}

wakeup_interrupt!(Spi0Wakeup, spi0);
wakeup_interrupt!(Spi1Wakeup, spi1);
wakeup_interrupt!(Usart0Wakeup, usart0);
wakeup_interrupt!(Usart1Wakeup, usart1);
wakeup_interrupt!(Usart2Wakeup, usart2);
wakeup_interrupt!(I2c1Wakeup, i2c1);
wakeup_interrupt!(I2c0Wakeup, i2c0);
wakeup_interrupt!(WwdtWakeup, wwdt);
wakeup_interrupt!(BodWakeup, bod);
wakeup_interrupt!(WktWakeup, wkt);
wakeup_interrupt!(I2c2Wakeup, i2c2);
wakeup_interrupt!(I2c3Wakeup, i2c3);

reg!(PDRUNCFG, PDRUNCFG, raw::SYSCON, pdruncfg);
#[cfg(feature = "82x")]
reg!(PRESETCTRL, PRESETCTRL, raw::SYSCON, presetctrl);
#[cfg(feature = "845")]
reg!(PRESETCTRL, PRESETCTRL, raw::SYSCON, presetctrl0);
reg!(STARTERP1, STARTERP1, raw::SYSCON, starterp1);
#[cfg(feature = "845")]
reg!(SYSAHBCLKCTRL, SYSAHBCLKCTRL, raw::SYSCON, sysahbclkctrl0);
#[cfg(feature = "82x")]
reg!(SYSAHBCLKCTRL, SYSAHBCLKCTRL, raw::SYSCON, sysahbclkctrl);
