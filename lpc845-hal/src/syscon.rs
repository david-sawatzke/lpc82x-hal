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
    impl_analog_block, AnalogBlock, BodWakeup, ClockControl, Handle, I2c0Wakeup, I2c1Wakeup,
    I2c2Wakeup, I2c3Wakeup, ResetControl, Spi0Wakeup, Spi1Wakeup, Usart0Wakeup, Usart1Wakeup,
    Usart2Wakeup, WktWakeup, WwdtWakeup, BOD, FLASH, MTB, RAM0_1, ROM, SYSOSC, SYSPLL,
};

use common::{clock, syscon::CommonParts};
use reg_proxy::RegProxy;

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
            fro: FRO(PhantomData),
            froout: FROOUT(PhantomData),
            mtb: parts.mtb,
            ram0_1: parts.ram0_1,
            rom: parts.rom,
            sysosc: parts.sysosc,
            syspll: parts.syspll,

            fro_derived_clock: FroDerivedClock::new(),
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

    /// FRO
    pub fro: FRO,

    /// FRO
    pub froout: FROOUT,

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

    /// The 750 kHz FRO-derived clock
    pub fro_derived_clock: FroDerivedClock<init_state::Enabled>,
}

/// FRO
///
/// Can be used to control FRO output using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct FRO(PhantomData<*const ()>);

/// FRO  output
///
/// Can be used to control FRO output using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct FROOUT(PhantomData<*const ()>);

/// The 750 kHz FRO-derived clock
///
/// This is one of the clocks that can be used to run the self-wake-up timer
/// (WKT). See user manual, section 23.5.1.
pub struct FroDerivedClock<State = init_state::Enabled> {
    _state: State,
}

impl FroDerivedClock<init_state::Enabled> {
    pub(crate) fn new() -> Self {
        FroDerivedClock {
            _state: init_state::Enabled(()),
        }
    }
}

impl FroDerivedClock<init_state::Disabled> {
    /// Enable the FRO-derived clock
    ///
    /// This method is only available, if `FroDerivedClock` is in the
    /// [`Disabled`] state. Code that attempts to call this method when the
    /// clock is already enabled will not compile.
    ///
    /// Consumes this instance of `FroDerivedClock` and returns another instance
    /// that has its `State` type parameter set to [`Enabled`]. That new
    /// instance implements [`clock::Enabled`], which might be required by APIs
    /// that need an enabled clock.
    ///
    /// Also consumes the handles to [`FRO`] and [`FROOUT`], to make it
    /// impossible (outside of unsafe code) to break API guarantees.
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [`clock::Enabled`]: ../clock/trait.Enabled.html
    pub fn enable(
        self,
        syscon: &mut Handle,
        mut fro: FRO,
        mut froout: FROOUT,
    ) -> FroDerivedClock<init_state::Enabled> {
        syscon.power_up(&mut fro);
        syscon.power_up(&mut froout);

        FroDerivedClock {
            _state: init_state::Enabled(()),
        }
    }
}

impl<State> clock::Frequency for FroDerivedClock<State> {
    fn hz(&self) -> u32 {
        750_000
    }
}

impl clock::Enabled for FroDerivedClock<init_state::Enabled> {}

impl_analog_block!(FRO, fro_pd);
impl_analog_block!(FROOUT, froout_pd);
