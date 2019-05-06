//! API for system configuration (SYSCON)
//!
//! The entry point to this API is [`SYSCON`]. Please refer to [`SYSCON`]'s
//! documentation for additional information.
//!
//! This module mostly provides infrastructure required by other parts of the
//! HAL API. For this reason, only a small subset of SYSCON functionality is
//! currently implemented.
//!
//! The SYSCON peripheral is described in the user manual, chapter 5.

use core::marker::PhantomData;

pub use crate::common::syscon::{
    AnalogBlock, BodWakeup, ClockControl, Handle, I2c0Wakeup, I2c1Wakeup, I2c2Wakeup, I2c3Wakeup,
    ResetControl, Spi0Wakeup, Spi1Wakeup, Usart0Wakeup, Usart1Wakeup, Usart2Wakeup, WktWakeup,
    WwdtWakeup, BOD, FLASH, MTB, RAM0_1, ROM, SYSOSC, SYSPLL,
};
use crate::raw::syscon::{pdruncfg, presetctrl, UARTCLKDIV, UARTFRGDIV, UARTFRGMULT};
use crate::{clock, common::syscon::CommonParts, init_state, raw, reg_proxy::RegProxy};

/// Entry point to the SYSCON API
///
/// The SYSCON API is split into multiple parts, which are all available through
/// [`syscon::Parts`]. You can use [`SYSCON::split`] to gain access to
/// [`syscon::Parts`].
///
/// You can also use this struct to gain access to the raw peripheral using
/// [`SYSCON::free`]. This is the main reason this struct exists, as it's no
/// longer possible to do this after the API has been split.
///
/// Use [`Peripherals`] to gain access to an instance of this struct.
///
/// Please refer to the [module documentation] for more information.
///
/// [`syscon::Parts`]: struct.Parts.html
/// [`Peripherals`]: ../struct.Peripherals.html
/// [module documentation]: index.html
pub struct SYSCON {
    syscon: raw::SYSCON,
}

impl SYSCON {
    pub(crate) fn new(syscon: raw::SYSCON) -> Self {
        SYSCON { syscon }
    }

    /// Splits the SYSCON API into its component parts
    ///
    /// This is the regular way to access the SYSCON API. It exists as an
    /// explicit step, as it's no longer possible to gain access to the raw
    /// peripheral using [`SYSCON::free`] after you've called this method.
    pub fn split(self) -> Parts {
        // NOTE(unsafe) We don't export this and it's only constructed
        // when splitting the syscon
        let parts = unsafe { CommonParts::new() };
        Parts {
            handle: parts.handle,
            bod: parts.bod,
            flash: parts.flash,
            irc: IRC(PhantomData),
            ircout: IRCOUT(PhantomData),
            mtb: parts.mtb,
            ram0_1: parts.ram0_1,
            rom: parts.rom,
            sysosc: parts.sysosc,
            syspll: parts.syspll,

            uartfrg: UARTFRG {
                uartclkdiv: RegProxy::new(),
                uartfrgdiv: RegProxy::new(),
                uartfrgmult: RegProxy::new(),
            },

            irc_derived_clock: IrcDerivedClock::new(),
        }
    }

    /// Return the raw peripheral
    ///
    /// This method serves as an escape hatch from the HAL API. It returns the
    /// raw peripheral, allowing you to do whatever you want with it, without
    /// limitations imposed by the API.
    ///
    /// If you are using this method because a feature you need is missing from
    /// the HAL API, please [open an issue] or, if an issue for your feature
    /// request already exists, comment on the existing issue, so we can
    /// prioritize it accordingly.
    ///
    /// [open an issue]: https://github.com/lpc-rs/lpc8xx-hal/issues
    pub fn free(self) -> raw::SYSCON {
        self.syscon
    }
}

/// The main API for the SYSCON peripheral
///
/// Provides access to all types that make up the SYSCON API. Please refer to
/// the [module documentation] for more information.
///
/// [module documentation]: index.html
pub struct Parts {
    /// The handle to the SYSCON peripheral
    pub handle: Handle,

    /// Brown-out detection
    pub bod: BOD,

    /// Flash memory
    pub flash: FLASH,

    /// IRC
    pub irc: IRC,

    /// IRC output
    pub ircout: IRCOUT,

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

    /// UART Fractional Baud Rate Generator
    pub uartfrg: UARTFRG,

    /// The 750 kHz IRC-derived clock
    pub irc_derived_clock: IrcDerivedClock<init_state::Enabled>,
}

/// IRC
///
/// Can be used to control the IRC using various methods on [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct IRC(PhantomData<*const ()>);

/// IRC output
///
/// Can be used to control IRC output using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct IRCOUT(PhantomData<*const ()>);

/// UART Fractional Baud Rate Generator
///
/// Controls the common clock for all UART peripherals (U_PCLK).
///
/// Can also be used to control the UART FRG using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct UARTFRG {
    uartclkdiv: RegProxy<UARTCLKDIV>,
    uartfrgdiv: RegProxy<UARTFRGDIV>,
    uartfrgmult: RegProxy<UARTFRGMULT>,
}

impl UARTFRG {
    /// Set UART clock divider value (UARTCLKDIV)
    ///
    /// See user manual, section 5.6.15.
    pub fn set_clkdiv(&mut self, value: u8) {
        self.uartclkdiv.write(|w| unsafe { w.div().bits(value) });
    }

    /// Set UART fractional generator multiplier value (UARTFRGMULT)
    ///
    /// See user manual, section 5.6.20.
    pub fn set_frgmult(&mut self, value: u8) {
        self.uartfrgmult.write(|w| unsafe { w.mult().bits(value) });
    }

    /// Set UART fractional generator divider value (UARTFRGDIV)
    ///
    /// See user manual, section 5.6.19.
    pub fn set_frgdiv(&mut self, value: u8) {
        self.uartfrgdiv.write(|w| unsafe { w.div().bits(value) });
    }
}

/// The 750 kHz IRC-derived clock
///
/// This is one of the clocks that can be used to run the self-wake-up timer
/// (WKT). See user manual, section 18.5.1.
pub struct IrcDerivedClock<State = init_state::Enabled> {
    _state: State,
}

impl IrcDerivedClock<init_state::Enabled> {
    pub(crate) fn new() -> Self {
        IrcDerivedClock {
            _state: init_state::Enabled(()),
        }
    }
}

impl IrcDerivedClock<init_state::Disabled> {
    /// Enable the IRC-derived clock
    ///
    /// This method is only available, if `IrcDerivedClock` is in the
    /// [`Disabled`] state. Code that attempts to call this method when the
    /// clock is already enabled will not compile.
    ///
    /// Consumes this instance of `IrcDerivedClock` and returns another instance
    /// that has its `State` type parameter set to [`Enabled`]. That new
    /// instance implements [`clock::Enabled`], which might be required by APIs
    /// that need an enabled clock.
    ///
    /// Also consumes the handles to [`IRC`] and [`IRCOUT`], to make it
    /// impossible (outside of unsafe code) to break API guarantees.
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [`clock::Enabled`]: ../clock/trait.Enabled.html
    pub fn enable(
        self,
        syscon: &mut Handle,
        mut irc: IRC,
        mut ircout: IRCOUT,
    ) -> IrcDerivedClock<init_state::Enabled> {
        syscon.power_up(&mut irc);
        syscon.power_up(&mut ircout);

        IrcDerivedClock {
            _state: init_state::Enabled(()),
        }
    }
}

impl<State> clock::Frequency for IrcDerivedClock<State> {
    fn hz(&self) -> u32 {
        750_000
    }
}

impl clock::Enabled for IrcDerivedClock<init_state::Enabled> {}

impl_reset_control!(UARTFRG, uartfrg_rst_n);

impl_analog_block!(IRCOUT, ircout_pd);
impl_analog_block!(IRC, irc_pd);

reg!(UARTCLKDIV, UARTCLKDIV, raw::SYSCON, uartclkdiv);
reg!(UARTFRGDIV, UARTFRGDIV, raw::SYSCON, uartfrgdiv);
reg!(UARTFRGMULT, UARTFRGMULT, raw::SYSCON, uartfrgmult);
