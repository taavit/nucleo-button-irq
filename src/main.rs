//! Turns the user LED on
//!
//! Listens for interrupts on the pa7 pin. On any rising or falling edge, toggles
//! the pc13 pin (which is connected to the LED on the blue pill board, hence the `led` name).

#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m_rt::entry;
use cortex_m::{asm::wfi, interrupt::Mutex};
use core::cell::RefCell;
use pac::interrupt;
use stm32f1xx_hal::gpio::*;
use stm32f1xx_hal::{pac, prelude::*};

type LedPin = gpioa::PA5<Output<PushPull>>;
type ButtonPin = gpioc::PC13<Input<Floating>>;

static G_LED: Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));

static G_INT_PIN: Mutex<RefCell<Option<ButtonPin>>> = Mutex::new(RefCell::new(None));

#[interrupt]
fn EXTI15_10() {
    static mut LED: Option<LedPin> = None;
    static mut INT_PIN: Option<ButtonPin> = None;

    let led = LED.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move LED pin here, leaving a None in its place
            G_LED.borrow(cs).replace(None).unwrap()
        })
    });

    let int_pin = INT_PIN.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move LED pin here, leaving a None in its place
            G_INT_PIN.borrow(cs).replace(None).unwrap()
        })
    });

    if int_pin.check_interrupt() {
        led.toggle();

        // if we don't clear this bit, the ISR would trigger indefinitely
        int_pin.clear_interrupt_pending_bit();
    }
}

#[entry]
fn main() -> ! {
    // initialization phase
    let mut p = pac::Peripherals::take().unwrap();
    // let _cp = cortex_m::peripheral::Peripherals::take().unwrap();
    {
        // the scope ensures that the int_pin reference is dropped before the first ISR can be executed.

        let mut gpioa = p.GPIOA.split();
        let mut gpioc = p.GPIOC.split();
        let mut afio = p.AFIO.constrain();

        let mut led = gpioa.pa5.into_push_pull_output(&mut gpioa.crl);
        let _ = led.set_high(); // Turn off

        cortex_m::interrupt::free(|cs| *G_LED.borrow(cs).borrow_mut() = Some(led));

        let mut int_pin = gpioc.pc13.into_floating_input(&mut gpioc.crh);
        int_pin.make_interrupt_source(&mut afio);
        int_pin.trigger_on_edge(&mut p.EXTI, Edge::RisingFalling);
        int_pin.enable_interrupt(&mut p.EXTI);

        cortex_m::interrupt::free(|cs| *G_INT_PIN.borrow(cs).borrow_mut() = Some(int_pin));
    } // initialization ends here

    unsafe {
        pac::NVIC::unmask(pac::Interrupt::EXTI15_10);
    }

    loop {
        wfi();
    }
}