#![no_main]
#![no_std]

use defmt_rtt as _; // global logger

use lis2mdl::Lis2mdl;
use stm32f4xx_hal as _;
use stm32f4xx_hal::gpio::GpioExt;
use stm32f4xx_hal::i2c::I2cExt;
use stm32f4xx_hal::pac;

use panic_probe as _;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::rcc::RccExt;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // Set up the LED. On the Nucleo-446RE it's connected to pin PA5.
        let mut led = dp.GPIOC.split().pc13.into_push_pull_output();

        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48_u32.MHz()).freeze();

        let mut delay = cp.SYST.delay(&clocks);

        let gpiob = dp.GPIOB.split();
        let scl = gpiob
            .pb6
            .into_alternate()
            .internal_pull_up(true)
            .set_open_drain();
        let sda = gpiob
            .pb7
            .into_alternate()
            .internal_pull_up(true)
            .set_open_drain();
        let i2c = dp.I2C1.i2c((scl, sda), 400.kHz(), &clocks);

        let mut lis = Lis2mdl::new(
            i2c,
            lis2mdl::DEFAULT_I2C_ADDRESS,
            lis2mdl::OutputDataRate::Hz10,
        )
        .unwrap();

        let id = lis.get_chip_id().unwrap();

        defmt::println!("magnetometer id: {}", id);

        loop {
            let mag = lis.read_magnetometer_data().unwrap();
            let temperature = lis.read_temperature().unwrap();
            defmt::println!("mag: {}, temperature: {}", mag, temperature);
            delay.delay_ms(100u32);
        }

        // Create a delay abstraction based on SysTick
        //let mut delay = cp.SYST.delay(&clocks);
    }

    loop {}
}

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

// defmt-test 0.3.0 has the limitation that this `#[tests]` attribute can only be used
// once within a crate. the module can be in any file but there can only be at most
// one `#[tests]` module in this library crate
#[cfg(test)]
#[defmt_test::tests]
mod unit_tests {
    use defmt::assert;

    #[test]
    fn it_works() {
        assert!(true)
    }
}
