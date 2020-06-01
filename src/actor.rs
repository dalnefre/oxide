use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::rc::Rc;
//use alloc::rc::Weak;
use core::cell::RefCell;
use alloc::collections::VecDeque;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::{print, println};

static mut ROOT: Option<Sponsor> = None;  // sponsor for interrupt handlers
static mut TIMER: Option<Rc<Actor>> = None;  // timer interrupt handler
static mut KEYBOARD: Option<Rc<Actor>> = None;  // keyboard interrupt handler

pub fn init() {
    let sponsor = Sponsor::new();
    let timer = sponsor.create(Box::new(tick_beh));
    let keyboard = sponsor.create(Box::new(keyboard_beh));
    unsafe {
        ROOT = Some(sponsor);
        TIMER = Some(timer);
        KEYBOARD = Some(keyboard);
    }
}

static mut TICKS: isize = 0;  // timer tick counter

/// WARNING! called directly from timer interrupt
pub unsafe fn notify_timer_actor() {
    if let Some(sponsor) = ROOT.as_ref() {
        if let Some(timer) = TIMER.as_ref() {
            let now = TICKS;
            TICKS += 1;
            sponsor.send(timer.clone(), &now);
        }
    }
}

/// safely obtain value of timer tick counter
pub fn now() -> isize {
    // we don't need a critical-section because
    // the timer interrupt makes incrementing appear atomic
    unsafe { return TICKS }
}

/// WARNING! called directly from keyboard interrupt
pub unsafe fn notify_keyboard_actor(scancode: u8) {
    if let Some(sponsor) = ROOT.as_ref() {
        if let Some(keyboard) = KEYBOARD.as_ref() {
            let code: Message = scancode as isize;
            sponsor.send(keyboard.clone(), &code);
        }
    }
}

/// safely obtain a reference to the root sponsor
pub fn root_sponsor() -> Option<&'static Sponsor> {
    unsafe { ROOT.as_ref() }
}

type Action = dyn Fn(&Event, &Sponsor) -> Effect;

type Message = isize; //Vec<u8>;

type Effect = bool;

pub struct Actor {
    behavior: RefCell<Box<Action>>,
}
impl Actor {
    fn new(behavior: Box<Action>) -> Actor {
        Actor {
            behavior: RefCell::new(behavior),
        }
    }
    fn react(&self, event: &Event, sponsor: &Sponsor) -> Effect {
        (self.behavior.borrow())(event, sponsor)
    }
}

pub struct Sponsor {
    actors: RefCell<Vec<Rc<Actor>>>,
    events: RefCell<VecDeque<Event>>,
}
impl Sponsor {
    pub fn new() -> Sponsor {
        Sponsor {
            actors: RefCell::new(Vec::new()),
            events: RefCell::new(VecDeque::new()),
        }
    }
    pub fn create(&self, behavior: Box<Action>) -> Rc<Actor> {
        let actor = Rc::new(Actor::new(behavior));
        self.actors.borrow_mut().push(Rc::clone(&actor));
        actor
    }
    pub fn send(&self, target: Rc<Actor>, message: &Message) {
        let event = Event::new(target, &message);
        x86_64::instructions::interrupts::without_interrupts(|| {
            self.events.borrow_mut().push_back(event);
        });
    }
    pub fn dispatch(&self) -> Effect {
        let next = x86_64::instructions::interrupts::without_interrupts(|| {
            self.events.borrow_mut().pop_front()
        });
        match next {
            Some(event) => event.dispatch(&self),
            None => false,  // no event to dispatch
        }
    }
}

pub struct Event {
    target: Rc<Actor>,
    message: Message,
}
impl Event{
    fn new(target: Rc<Actor>, message: &Message) -> Event {
        Event {
            target,
            message: message.clone(),
        }
    }
    fn dispatch(&self, sponsor: &Sponsor) -> Effect {
        self.target.react(&self, &sponsor)
        // FIXME: should return an Ok/Fail indication like Result or Option
    }
}

/// No-Op actor behavior
pub fn sink_beh(_event: &Event, _sponsor: &Sponsor) -> Effect {
    true
}

/// Timer tick-counter actor behavior
pub fn tick_beh(event: &Event, _sponsor: &Sponsor) -> Effect {
    use crate::vga_screen::stat_print;

    //print!("{}", event.message);
    //print!("'");
    let status = [b'/', b'-', b'\\', b'|'];
    let index = (event.message & 0x03) as usize;
    stat_print(status[index]);
    true
}

/// Keyboard key-translation actor behavior
pub fn keyboard_beh(event: &Event, _sponsor: &Sponsor) -> Effect {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
        );
    }

    let scancode: u8 = event.message as u8;
    //print!("<{}>", scancode);

    let mut keyboard = KEYBOARD.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    true
}

/// Debug printing actor behavior
pub fn debug_beh(event: &Event, _sponsor: &Sponsor) -> Effect {
    println!("--DEBUG-- message={:x}", &event.message);
    true
}

pub fn try_actors() {
    println!("> try_actors");
    if let Some(sponsor) = root_sponsor() {
        let a_debug = sponsor.create(Box::new(debug_beh));
        let message: isize = 0xC0FFEEFACADE;
        sponsor.send(a_debug, &message);
    }
    println!("< try_actors");
}

/// Root sponsor dispatch loop
///
/// wait-for-interrupt when there are no events to dispatch
pub fn dispatch_loop() -> ! {
    let sponsor = root_sponsor().unwrap();
    loop {
        //println!("It did not crash! (yet)");
        if sponsor.dispatch() == false {
            x86_64::instructions::hlt();  // no more events, so wait for interrupt...
        }
    }
}
