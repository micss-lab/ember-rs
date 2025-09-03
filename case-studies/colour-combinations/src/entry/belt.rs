use alloc::{collections::VecDeque, rc::Rc};
use core::cell::RefCell;

use super::Colour;

#[derive(Clone)]
pub struct Belt(Rc<RefCell<BeltInner>>);

impl Belt {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Colour>,
    {
        Self(Rc::new(RefCell::new(BeltInner::new(iter))))
    }

    pub fn take_next(&self) -> Option<Colour> {
        self.borrow_mut().take_next()
    }

    pub fn peek_next(&self) -> Option<Colour> {
        self.borrow().peek_next()
    }

    pub fn next_window(&self) -> Option<Window> {
        self.borrow_mut().next_window()
    }

    pub fn made_combination(&self, bottom: Colour, top: Colour) -> usize {
        self.borrow_mut().made_combination(bottom, top)
    }

    pub fn print_score(&self) {
        self.borrow().print_score();
    }
}

impl core::ops::Deref for Belt {
    type Target = <Rc<RefCell<BeltInner>> as core::ops::Deref>::Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct BeltInner {
    items: VecDeque<Colour>,
    score: usize,
}

pub struct Window {
    pub first: Colour,
    pub second: Option<Colour>,
}

impl From<(Colour, Option<Colour>)> for Window {
    fn from((first, second): (Colour, Option<Colour>)) -> Self {
        Self { first, second }
    }
}

impl BeltInner {
    fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Colour>,
    {
        Self::from_iter(iter)
    }

    fn take_next(&mut self) -> Option<Colour> {
        self.items.pop_front()
    }

    fn peek_next(&self) -> Option<Colour> {
        self.items.front().copied()
    }

    fn next_window(&mut self) -> Option<Window> {
        self.take_next().map(|c1| {
            let c2 = self.peek_next();
            self.items.push_front(c1);
            Window::from((c1, c2))
        })
    }

    fn made_combination(&mut self, bottom: Colour, top: Colour) -> usize {
        let val = match (bottom, top) {
            (Colour::Red, Colour::Red) => 100,
            (Colour::Red, _) | (_, Colour::Red) => 50,
            (Colour::Green, Colour::Green) | (Colour::Blue, Colour::Blue) => 25,
            _ => 0,
        };
        self.score += val;
        return val;
    }

    fn print_score(&self) {
        log::info!("Final score: {}", self.score);
    }
}

impl FromIterator<Colour> for BeltInner {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Colour>,
    {
        Self {
            items: VecDeque::from_iter(iter).into(),
            score: 0,
        }
    }
}
