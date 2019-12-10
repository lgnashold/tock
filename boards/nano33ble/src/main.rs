//! Tock kernel for the Arduino Nano 33 BLE.
//!
//! It is based on nRF52840 SoC (Cortex M4 core with a BLE transceiver).

#![no_std]
#![no_main]
#![deny(missing_docs)]

use capsules::led::ActivationMode::ActiveLow;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, debug_verbose, static_init};

use nrf52840::gpio::Pin;

// Three-color LED.
const LED_RED_PIN: Pin = Pin::P0_24;
const LED_GREEN_PIN: Pin = Pin::P0_16;
const LED_BLUE_PIN: Pin = Pin::P0_06;

const BUTTON_RST_PIN: Pin = Pin::P0_18;

const GPIO_D2: Pin = Pin::P1_11;
const GPIO_D3: Pin = Pin::P1_12;
const GPIO_D4: Pin = Pin::P1_15;
const GPIO_D5: Pin = Pin::P1_13;
const GPIO_D6: Pin = Pin::P1_14;
const GPIO_D7: Pin = Pin::P0_23;
const GPIO_D8: Pin = Pin::P0_21;
const GPIO_D9: Pin = Pin::P0_27;
const GPIO_D10: Pin = Pin::P1_02;

const UART_TX_PIN: Pin = Pin::P1_03;
const UART_RX_PIN: Pin = Pin::P1_10;

/// UART Writer for panic!()s.
pub mod io;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 245760] = [0; 245760];

static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None, None, None, None, None, None];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// Supported drivers by the platform
pub struct Platform {
    // ble_radio: &'static capsules::ble_advertising_driver::BLE<
    //     'static,
    //     nrf52::ble_radio::Radio,
    //     VirtualMuxAlarm<'static, Rtc<'static>>,
    // >,
    // ieee802154_radio: Option<&'static capsules::ieee802154::RadioDriver<'static>>,
    console: &'static capsules::console::Console<'static>,
    gpio: &'static capsules::gpio::GPIO<'static>,
    led: &'static capsules::led::LED<'static>,
    // rng: &'static capsules::rng::RngDriver<'static>,
    ipc: kernel::ipc::IPC,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
    >,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            // capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            // capsules::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            // capsules::ieee802154::DRIVER_NUM => match self.ieee802154_radio {
            //     Some(radio) => f(Some(radio)),
            //     None => f(None),
            // },
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

/// Entry point in the vector table called on hard reset.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Loads relocations and clears BSS
    nrf52840::init();

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    // Make non-volatile memory writable and activate the reset button
    // let uicr = nrf52::uicr::Uicr::new();

    // // Check if we need to erase UICR memory to re-program it
    // // This only needs to be done when a bit needs to be flipped from 0 to 1.
    // let psel0_reset: u32 = uicr.get_psel0_reset_pin().map_or(0, |pin| pin as u32);
    // let psel1_reset: u32 = uicr.get_psel1_reset_pin().map_or(0, |pin| pin as u32);
    // let erase_uicr = ((!psel0_reset & (BUTTON_RST_PIN as u32))
    //     | (!psel1_reset & (BUTTON_RST_PIN as u32))
    //     | (!(uicr.get_vout() as u32) & (nrf52840::uicr::Regulator0Output::DEFAULT as u32)))
    //     != 0;

    // if erase_uicr {
    //     nrf52::nvmc::NVMC.erase_uicr();
    // }

    // nrf52::nvmc::NVMC.configure_writeable();
    // while !nrf52::nvmc::NVMC.is_ready() {}

    // let mut needs_soft_reset: bool = false;

    // // Configure reset pins
    // if uicr
    //     .get_psel0_reset_pin()
    //     .map_or(true, |pin| pin != BUTTON_RST_PIN)
    // {
    //     uicr.set_psel0_reset_pin(BUTTON_RST_PIN);
    //     while !nrf52::nvmc::NVMC.is_ready() {}
    //     needs_soft_reset = true;
    // }
    // if uicr
    //     .get_psel1_reset_pin()
    //     .map_or(true, |pin| pin != BUTTON_RST_PIN)
    // {
    //     uicr.set_psel1_reset_pin(BUTTON_RST_PIN);
    //     while !nrf52::nvmc::NVMC.is_ready() {}
    //     needs_soft_reset = true;
    // }

    // // Configure voltage regulator output
    // if uicr.get_vout() != nrf52840::uicr::Regulator0Output::DEFAULT {
    //     uicr.set_vout(nrf52840::uicr::Regulator0Output::DEFAULT);
    //     while !nrf52::nvmc::NVMC.is_ready() {}
    //     needs_soft_reset = true;
    // }

    // // Any modification of UICR needs a soft reset for the changes to be taken into account.
    // if needs_soft_reset {
    //     cortexm4::scb::reset();
    // }

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&nrf52840::gpio::PORT[LED_RED_PIN]),
        Some(&nrf52840::gpio::PORT[LED_GREEN_PIN]),
        Some(&nrf52840::gpio::PORT[LED_BLUE_PIN]),
    );

    //--------------------------------------------------------------------------
    // GPIO
    //--------------------------------------------------------------------------

    let gpio = components::gpio::GpioPinsNine::new(
        &nrf52840::gpio::PORT[GPIO_D2],
        &nrf52840::gpio::PORT[GPIO_D3],
        &nrf52840::gpio::PORT[GPIO_D4],
        &nrf52840::gpio::PORT[GPIO_D5],
        &nrf52840::gpio::PORT[GPIO_D6],
        &nrf52840::gpio::PORT[GPIO_D7],
        &nrf52840::gpio::PORT[GPIO_D8],
        &nrf52840::gpio::PORT[GPIO_D9],
        &nrf52840::gpio::PORT[GPIO_D10],
        board_kernel,
    )
    .finalize(());

    //--------------------------------------------------------------------------
    // LEDs
    //--------------------------------------------------------------------------

    let led = components::led::LedsThree::new(
        (&nrf52840::gpio::PORT[LED_RED_PIN], ActiveLow),
        (&nrf52840::gpio::PORT[LED_GREEN_PIN], ActiveLow),
        (&nrf52840::gpio::PORT[LED_BLUE_PIN], ActiveLow),
    )
    .finalize(());

    use kernel::hil::gpio::Configure;
    use kernel::hil::gpio::Output;
    &nrf52840::gpio::PORT[LED_RED_PIN].make_output();
    &nrf52840::gpio::PORT[LED_RED_PIN].clear();
    &nrf52840::gpio::PORT[LED_GREEN_PIN].make_output();
    &nrf52840::gpio::PORT[LED_GREEN_PIN].set();
    &nrf52840::gpio::PORT[LED_BLUE_PIN].make_output();
    &nrf52840::gpio::PORT[LED_BLUE_PIN].clear();

    //--------------------------------------------------------------------------
    // ALARM & TIMER
    //--------------------------------------------------------------------------

    let rtc = &nrf52::rtc::RTC;
    rtc.start();

    let mux_alarm = components::alarm::AlarmMuxComponent::new(rtc)
        .finalize(components::alarm_mux_component_helper!(nrf52::rtc::Rtc));
    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
        .finalize(components::alarm_component_helper!(nrf52::rtc::Rtc));

    //--------------------------------------------------------------------------
    // UART & CONSOLE & DEBUG
    //--------------------------------------------------------------------------

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(&nrf52::uart::UARTE0, 115200).finalize(());

    // Configure the UART pins on this specific board.
    nrf52::uart::UARTE0.initialize(
        nrf52::pinmux::Pinmux::new(UART_TX_PIN as u32),
        nrf52::pinmux::Pinmux::new(UART_RX_PIN as u32),
        None,
        None,
    );

    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    // let ble_radio =
    //     BLEComponent::new(board_kernel, &nrf52::ble_radio::RADIO, mux_alarm).finalize(());

    // let ieee802154_radio = if ieee802154 {
    //     let (radio, _) = Ieee802154Component::new(
    //         board_kernel,
    //         &nrf52::ieee802154_radio::RADIO,
    //         PAN_ID,
    //         SRC_MAC,
    //     )
    //     .finalize(());
    //     Some(radio)
    // } else {
    //     None
    // };

    // let temp = static_init!(
    //     capsules::temperature::TemperatureSensor<'static>,
    //     capsules::temperature::TemperatureSensor::new(
    //         &nrf52::temperature::TEMP,
    //         board_kernel.create_grant(&memory_allocation_capability)
    //     )
    // );
    // kernel::hil::sensors::TemperatureDriver::set_client(&nrf52::temperature::TEMP, temp);

    // let rng = components::rng::RngComponent::new(board_kernel, &nrf52::trng::TRNG).finalize(());

    // // SPI
    // let mux_spi = static_init!(
    //     MuxSpiMaster<'static, nrf52::spi::SPIM>,
    //     MuxSpiMaster::new(&nrf52::spi::SPIM0)
    // );
    // hil::spi::SpiMaster::set_client(&nrf52::spi::SPIM0, mux_spi);
    // hil::spi::SpiMaster::init(&nrf52::spi::SPIM0);
    // nrf52::spi::SPIM0.configure(
    //     nrf52::pinmux::Pinmux::new(spi_pins.mosi as u32),
    //     nrf52::pinmux::Pinmux::new(spi_pins.miso as u32),
    //     nrf52::pinmux::Pinmux::new(spi_pins.clk as u32),
    // );

    // let nonvolatile_storage: Option<
    //     &'static capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>,
    // > = if let Some(driver) = mx25r6435f {
    //     // Create a SPI device for the mx25r6435f flash chip.
    //     let mx25r6435f_spi = static_init!(
    //         capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>,
    //         capsules::virtual_spi::VirtualSpiMasterDevice::new(
    //             mux_spi,
    //             &gpio_port[driver.chip_select]
    //         )
    //     );
    //     // Create an alarm for this chip.
    //     let mx25r6435f_virtual_alarm = static_init!(
    //         VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
    //         VirtualMuxAlarm::new(mux_alarm)
    //     );
    //     // Setup the actual MX25R6435F driver.
    //     let mx25r6435f = static_init!(
    //         capsules::mx25r6435f::MX25R6435F<
    //             'static,
    //             capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>,
    //             nrf52::gpio::GPIOPin,
    //             VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
    //         >,
    //         capsules::mx25r6435f::MX25R6435F::new(
    //             mx25r6435f_spi,
    //             mx25r6435f_virtual_alarm,
    //             &mut capsules::mx25r6435f::TXBUFFER,
    //             &mut capsules::mx25r6435f::RXBUFFER,
    //             Some(&gpio_port[driver.write_protect_pin]),
    //             Some(&gpio_port[driver.hold_pin])
    //         )
    //     );
    //     mx25r6435f_spi.set_client(mx25r6435f);
    //     hil::time::Alarm::set_client(mx25r6435f_virtual_alarm, mx25r6435f);

    //     pub static mut FLASH_PAGEBUFFER: capsules::mx25r6435f::Mx25r6435fSector =
    //         capsules::mx25r6435f::Mx25r6435fSector::new();
    //     let nv_to_page = static_init!(
    //         capsules::nonvolatile_to_pages::NonvolatileToPages<
    //             'static,
    //             capsules::mx25r6435f::MX25R6435F<
    //                 'static,
    //                 capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>,
    //                 nrf52::gpio::GPIOPin,
    //                 VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
    //             >,
    //         >,
    //         capsules::nonvolatile_to_pages::NonvolatileToPages::new(
    //             mx25r6435f,
    //             &mut FLASH_PAGEBUFFER
    //         )
    //     );
    //     hil::flash::HasClient::set_client(mx25r6435f, nv_to_page);

    //     let nonvolatile_storage = static_init!(
    //         capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>,
    //         capsules::nonvolatile_storage_driver::NonvolatileStorage::new(
    //             nv_to_page,
    //             board_kernel.create_grant(&memory_allocation_capability),
    //             0x60000, // Start address for userspace accessible region
    //             0x20000, // Length of userspace accessible region
    //             0,       // Start address of kernel accessible region
    //             0x60000, // Length of kernel accessible region
    //             &mut capsules::nonvolatile_storage_driver::BUFFER
    //         )
    //     );
    //     hil::nonvolatile_storage::NonvolatileStorage::set_client(nv_to_page, nonvolatile_storage);
    //     Some(nonvolatile_storage)
    // } else {
    //     None
    // };

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    nrf52::clock::CLOCK.low_stop();
    nrf52::clock::CLOCK.high_stop();

    nrf52::clock::CLOCK.low_set_source(nrf52::clock::LowClockSource::XTAL);
    nrf52::clock::CLOCK.low_start();
    nrf52::clock::CLOCK.high_set_source(nrf52::clock::HighClockSource::XTAL);
    nrf52::clock::CLOCK.high_start();
    while !nrf52::clock::CLOCK.low_started() {}
    while !nrf52::clock::CLOCK.high_started() {}

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 1], Default::default());
    let dynamic_deferred_call = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_call);

    let platform = Platform {
        // ble_radio: ble_radio,
        // ieee802154_radio: ieee802154_radio,
        console: console,
        led: led,
        gpio: gpio,
        // rng: rng,
        alarm: alarm,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
    };

    let chip = static_init!(
        nrf52::chip::NRF52,
        nrf52::chip::NRF52::new(&nrf52840::gpio::PORT)
    );

    // debug!("Initialization complete. Entering main loop\r");
    // debug!("{}", &nrf52::ficr::FICR_INSTANCE);

    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }
    kernel::procs::load_processes(
        board_kernel,
        chip,
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_management_capability,
    );

    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);

    // nrf52dk_base::setup_board(
    //     board_kernel,
    //     BUTTON_RST_PIN,
    //     &nrf52840::gpio::PORT,
    //     gpio_pins,
    //     LED1_PIN,
    //     LED2_PIN,
    //     LED3_PIN,
    //     led_pins,
    //     &UartPins::new(UART_RTS, UART_TXD, UART_CTS, UART_RXD),
    //     &SpiPins::new(SPI_MOSI, SPI_MISO, SPI_CLK),
    //     &Some(SpiMX25R6435FPins::new(
    //         SPI_MX25R6435F_CHIP_SELECT,
    //         SPI_MX25R6435F_WRITE_PROTECT_PIN,
    //         SPI_MX25R6435F_HOLD_PIN,
    //     )),
    //     button_pins,
    //     true,
    //     &mut APP_MEMORY,
    //     &mut PROCESSES,
    //     FAULT_RESPONSE,
    //     nrf52840::uicr::Regulator0Output::DEFAULT,
    //     false,
    // );
}
