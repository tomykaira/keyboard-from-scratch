use stm32l4::stm32l4x2::Peripherals;
use stm32l4xx_hal::hal::watchdog::WatchdogEnable;
use stm32l4xx_hal::time::MilliSeconds;
use stm32l4xx_hal::watchdog::IndependentWatchdog;

const KEY1: u32 = 0x45670123;
const KEY2: u32 = 0xCDEF89AB;
const OPTKEY1: u32 = 0x08192A3B;
const OPTKEY2: u32 = 0x4C5D6E7F;

pub unsafe fn reset() {
    // Restart by watchdog.
    let mut wd = IndependentWatchdog::new(p.IWDG);
    wd.start(MilliSeconds(1));

    // Wait asynchronous reset.
    loop {}
}
