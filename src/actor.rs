use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::rc::Rc;
//use alloc::rc::Weak;
use core::cell::RefCell;
use alloc::collections::VecDeque;
//use x86_64::instructions::interrupts;
use crate::{print, println};

static mut ROOT: Option<Sponsor> = None;  // sponsor for interrupt handlers
static mut TIMER: Option<Rc<Actor>> = None;  // timer interrupt handler

pub fn init() {
    let sponsor = Sponsor::new();
    let timer = sponsor.create(Box::new(tick_beh), &Vec::new());
    unsafe {
        ROOT = Some(sponsor);
        TIMER = Some(timer);
    }
}

pub unsafe fn notify_timer_actor() {  // WARNING! called directly from timer interrupt
    if let Some(sponsor) = ROOT.as_ref() {
        if let Some(timer) = TIMER.as_ref() {
            //print!("<>");
            sponsor.send(timer.clone(), &42);
        }
    }
}

pub fn root_sponsor() -> Option<&'static Sponsor> {  // safely obtain a reference to the root sponsor
    unsafe { ROOT.as_ref() }
}

type Action = dyn Fn(&Event, &Sponsor) -> Effect;

type Message = usize; //Vec<u8>;

type State = Vec<u8>;

type Effect = bool;

pub struct Actor {
    behavior: RefCell<Box<Action>>,
    state: RefCell<State>,
}
impl Actor {
    fn new(behavior: Box<Action>, state: &State) -> Actor {
        Actor {
            behavior: RefCell::new(behavior),
            state: RefCell::new(state.clone()),
        }
    }
    fn notice(&self, event: &Event, sponsor: &Sponsor) -> Effect {
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
    pub fn create(&self, behavior: Box<Action>, state: &State) -> Rc<Actor> {
        let actor = Rc::new(Actor::new(behavior, &state));
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
        self.target.notice(&self, &sponsor)
        // FIXME: should return an Ok/Fail indication like Result or Option
    }
}

pub fn sink_beh(_event: &Event, _sponsor: &Sponsor) -> Effect {
    // no-op
    true
}

pub fn tick_beh(_event: &Event, _sponsor: &Sponsor) -> Effect {
    // timer tick handler
    print!("'");
    true
}

pub fn debug_beh(event: &Event, _sponsor: &Sponsor) -> Effect {
    println!("--DEBUG-- state:{:?} message:{:?}", &event.target.state, &event.message);
    true
}

fn str_as_vec(s: &str) -> Vec<u8> {
    let mut v = Vec::new();
    for b in s.bytes() {
        v.push(b)
    }
    v
}

pub fn try_actors() {
    println!("> try_actors");
    if let Some(sponsor) = root_sponsor() {
        let a_debug = sponsor.create(Box::new(debug_beh), &str_as_vec("Hello"));
        let message: usize = 0xC0FFEEFACADE;
        sponsor.send(a_debug, &message);
    }
    println!("< try_actors");
}

pub fn dispatch_loop() -> ! {
    println!("> dispatch_loop");
    let sponsor = root_sponsor().unwrap();
    loop {
        if sponsor.dispatch() == false {
            x86_64::instructions::hlt();  // no more events, so wait for interrupt...
        }
    }
    //println!("< dispatch_loop");
}
