#![no_main]
#![no_std]

use rtt_target::{rtt_init_print, rprintln};                                   
use panic_rtt_target as _;                                                    

use core::cell::RefCell;

use cortex_m::{
    asm,
    interrupt::Mutex,
};
use cortex_m_rt::entry;
use microbit::{
    board::Board,
    hal::gpio::{Pin, Input, Floating},
    hal::gpiote::{Gpiote, GpioteChannel},
    pac::{self, interrupt},
};

static GPIO: Mutex<RefCell<Option<Gpiote>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();

    let gpiote = Gpiote::new(board.GPIOTE);

    let setup_channel = |channel: GpioteChannel, button: &Pin<Input<Floating>>| {
        channel
            .input_pin(button)
            .hi_to_lo()
            .enable_interrupt();
        channel.reset_events();
    };

    setup_channel(gpiote.channel0(), &board.buttons.button_a.degrade());
    setup_channel(gpiote.channel1(), &board.buttons.button_b.degrade());

    cortex_m::interrupt::free(move |cs| {
        /* Enable external GPIO interrupts */
        unsafe {
            pac::NVIC::unmask(pac::Interrupt::GPIOTE);
        }
        pac::NVIC::unpend(pac::Interrupt::GPIOTE);

        *GPIO.borrow(cs).borrow_mut() = Some(gpiote);

        rprintln!("Welcome to the buttons demo. Press buttons A and/or B for some action.");
    });

    loop {
        asm::wfi();
        rprintln!("got interrupt");
    }
}

// Define an interrupt, i.e. function to call when exception occurs. Here if we receive an
// interrupt from a button press, the function will be called
#[interrupt]
fn GPIOTE() {
    /* Enter critical section */
    cortex_m::interrupt::free(|cs| {
        rprintln!("interrupt");
        if let Some(gpiote) = GPIO.borrow(cs).borrow().as_ref() {
            let buttonapressed = gpiote.channel0().is_event_triggered();
            let buttonbpressed = gpiote.channel1().is_event_triggered();

            /* Print buttons to the serial console */
            rprintln!(
                "button pressed {:?}",
                match (buttonapressed, buttonbpressed) {
                    (false, false) => "",
                    (true, false) => "A",
                    (false, true) => "B",
                    (true, true) => "A + B",
                }
            );

            /* Clear events */
            gpiote.channel0().reset_events();
            gpiote.channel1().reset_events();
        }
    });
}
