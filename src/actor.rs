use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::rc::Rc;
//use alloc::rc::Weak;
use core::cell::RefCell;
use alloc::collections::VecDeque;
use crate::println;

type Action = dyn Fn(&Event, &Sponsor) -> Option<usize>;

type Message = Vec<u8>;

type State = Vec<u8>;

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
	fn notice(&self, event: &Event, sponsor: &Sponsor) -> Option<usize> {
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
	pub fn send(&self, target: Rc<Actor>, message: &Message) -> bool {
		let event = Event::new(target, &message);
		self.events.borrow_mut().push_back(event);
		true
	}
	pub fn dispatch(&self) -> Option<usize> {
		match self.events.borrow_mut().pop_front() {
			Some(event) => event.dispatch(&self),
			None => Some(1)  // FIXME: this is an error code, refactor to use Result<T,E>?
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
	fn dispatch(&self, sponsor: &Sponsor) -> Option<usize> {
		self.target.notice(&self, &sponsor)
	}
}

pub fn sink_beh(_event: &Event, _sponsor: &Sponsor) -> Option<usize> {
	// no-op
	None
}

pub fn debug_beh(event: &Event, _sponsor: &Sponsor) -> Option<usize> {
	println!("--DEBUG-- state:{:?} message:{:?}", &event.target.state, &event.message);
	None
}

fn str_to_vec(s: &str) -> Vec<u8> {
	let mut v = Vec::new();
	for b in s.bytes() {
		v.push(b)
	}
	v
}

pub fn try_actors() {
	println!("> try_actors");
	let sponsor = Sponsor::new();
	let a_debug = sponsor.create(Box::new(debug_beh), &str_to_vec("Hello"));
	sponsor.send(a_debug, &str_to_vec("World"));
	match sponsor.dispatch() {
		None => println!("OK."),
		Some(errno) => println!("FAIL! ({})", errno),
	}
	assert_eq!(sponsor.dispatch(), Some(1));
	println!("< try_actors");
}

/*
 * Sponsor Interface
   * event:Event
     * sponsor:Sponsor
     * target:Address
       * behavior:Script
       * state:Env
       * scope:Env
     * message:Env
   * ok{}:⊥ -- maybe take behavior and act like become?
   * fail{err:Expr->Data}:⊥
   * create{key:Expr->Data, behavior:Expr->Script, state:Expr->Env, scope:Expr->Env}
   * send{target:Expr->Address, message:Expr->Env}
   * assign{key:Expr->Data, value:Expr->Data}
   * become{behavior:Expr->Script}
   * die{} -- this actor will not receive any more messages
 * Rust Types?
   * Actor { notice(Event) -> Effect }
   * Event { sponsor: Sponsor, target: Actor; message: Message }
   * Effect { actors:Vec<Actor>, events:Vec<Event>, state: State, become: Behavior, error:Option<Error> }
*/
