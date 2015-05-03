//          Copyright Corey Richardson 2015
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_1_0.txt or copy at
//          http://www.boost.org/LICENSE_1_0.txt)

//! A real-time hierarchical profiler.

#[macro_use]
extern crate log;
extern crate clock_ticks;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// A single tree of profile data.
pub struct Profiler {
    root: Rc<ProfileNode>,
    current: RefCell<Rc<ProfileNode>>,
    enabled: Cell<bool>,
}

/// A "guard" for calling `Profiler::leave` when it is destroyed.
pub struct ProfileGuard<'a>(&'a Profiler);
impl<'a> Drop for ProfileGuard<'a> {
    fn drop(&mut self) {
        self.0.leave()
    }
}

macro_rules! early_leave {
    ($slf:ident) => (if $slf.enabled.get() == false { return })
}

impl Profiler {
    /// Create a new profiler with the given name for the root node.
    pub fn new(name: &'static str) -> Profiler {
        let root = Rc::new(ProfileNode::new(None, name));
        root.call();
        Profiler { root: root.clone(), current: RefCell::new(root), enabled: Cell::new(true) }
    }

    /// Enter a profile node for `name`, returning a guard object that will `leave` on destruction.
    pub fn enter(&self, name: &'static str) -> ProfileGuard {
        self.enter_noguard(name);
        ProfileGuard(self)
    }

    /// Enter a profile node for `name`.
    pub fn enter_noguard(&self, name: &'static str) {
        early_leave!(self);
        {
            let mut curr = self.current.borrow_mut();
            if curr.name != name {
                *curr = curr.make_child(curr.clone(), name);
            }
        }
        self.current.borrow().call();
    }

    /// Leave the current profile node.
    pub fn leave(&self) {
        early_leave!(self);
        let mut curr = self.current.borrow_mut();
        curr.ret();
        if let Some(parent) = curr.parent.clone() {
            *curr = parent;
        }
    }

    /// Print out the current timing information in a very naive way.
    pub fn print_timing(&self) {
        println!("Timing information for {}:", self.root.name);
        for child in &*self.root.children.borrow() {
            child.print(2);
        }
    }

    /// Return the root profile node for inspection.
    ///
    /// This root will always be valid and reflect the current state of the `Profiler`.
    /// It is not advised to inspect the data between calls to `start_frame` and `end_frame`.
    pub fn root(&self) -> Rc<ProfileNode> {
        self.root.clone()
    }

    /// Finish a frame.
    ///
    /// Logs an error if there are pending `leave` calls, and later attempts to
    /// print timing data will be met with sadness in the form of `NaN`s.
    pub fn end_frame(&self) {
        early_leave!(self);
        if &*self.root as *const ProfileNode as usize != &**self.current.borrow() as *const ProfileNode as usize {
            error!("Pending `leave` calls on Profiler::frame");
        } else {
            self.root.ret();
        }
    }

    /// Start a frame.
    ///
    /// Resets timing data. Logs an error if there are pending `leave` calls, but there are
    /// otherwise no ill effects.
    pub fn start_frame(&self) {
        early_leave!(self);
        if &*self.root as *const ProfileNode as usize != &**self.current.borrow() as *const ProfileNode as usize {
            error!("Pending `leave` calls on Profiler::frame");
        }
        *self.current.borrow_mut() = self.root.clone();
        self.root.reset();
        self.root.call();
    }

    /// Disable the profiler.
    ///
    /// All calls until `enable` will do nothing.
    pub fn disable(&self) {
        self.enabled.set(false);
    }

    /// Enable the profiler.
    ///
    /// Calls will take effect until `disable` is called.
    pub fn enable(&self) {
        self.enabled.set(true);
    }

    /// Toggle the profiler enabledness.
    pub fn toggle(&self) {
        self.enabled.set(!self.enabled.get());
    }

}

/// A single node in the profile tree.
///
/// *NOTE*: While the fields are public and are a cell, it is not advisable to modify them.
pub struct ProfileNode {
    pub name: &'static str,
    /// Number of calls made to this node.
    pub calls: Cell<u32>,
    /// Total time in ns used by this node and all of its children.
    ///
    /// Computed after the last pending `ret`.
    pub total_time: Cell<u64>,
    /// Timestamp in ns when the first `call` was made to this node.
    pub start_time: Cell<u64>,
    /// Number of recursive calls made to this node since the first `call`.
    pub recursion: Cell<u32>,
    /// Parent in the profile tree.
    pub parent: Option<Rc<ProfileNode>>,
    // TODO: replace this Vec with an intrusive list. Use containerof?
    /// Child nodes.
    pub children: RefCell<Vec<Rc<ProfileNode>>>,
}

impl ProfileNode {
    pub fn new(parent: Option<Rc<ProfileNode>>, name: &'static str) -> ProfileNode {
        ProfileNode {
            name: name,
            calls: Cell::new(0),
            total_time: Cell::new(0),
            start_time: Cell::new(0),
            recursion: Cell::new(0),
            parent: parent,
            children: RefCell::new(Vec::new())
        }
    }

    /// Reset this node and its children, seting relevant fields to 0.
    pub fn reset(&self) {
        self.calls.set(0);
        self.total_time.set(0);
        self.start_time.set(0);
        self.recursion.set(0);
        for child in &*self.children.borrow() {
            child.reset()
        }
    }

    /// Create a child named `name`.
    pub fn make_child(&self, me: Rc<ProfileNode>, name: &'static str) -> Rc<ProfileNode> {
        let mut children = self.children.borrow_mut();
        for child in &*children {
            if child.name == name {
                return child.clone()
            }
        }
        let new = Rc::new(ProfileNode::new(Some(me), name));
        children.push(new.clone());
        new
    }

    /// Enter this profile node.
    pub fn call(&self) {
        self.calls.set(self.calls.get() + 1);
        let rec = self.recursion.get();
        if rec == 0 {
            self.start_time.set(clock_ticks::precise_time_ns());
        }
        self.recursion.set(rec + 1);
    }

    /// Return from this profile node, returning true if there are no pending recursive calls.
    pub fn ret(&self) -> bool {
        let rec = self.recursion.get();
        if rec == 1 {
            let time = clock_ticks::precise_time_ns();
            let durr = time - self.start_time.get();
            self.total_time.set(self.total_time.get() + durr);
        }
        self.recursion.set(rec - 1);
        rec == 1
    }

    /// Print out the current timing information in a very naive way.
    ///
    /// Uses `indent` to determine how deep to indent the line.
    pub fn print(&self, indent: u32) {
        for _ in 0..indent {
            print!(" ");
        }
        println!("{} - {}ns ({}%)", self.name, self.total_time.get(), 100.0 * (self.total_time.get() as f64 / self.parent.as_ref().map(|p| p.total_time.get()).unwrap_or(self.total_time.get()) as f64));
        for c in &*self.children.borrow() {
            c.print(indent+2);
        }
    }
}
