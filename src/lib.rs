//          Copyright Corey Richardson 2015
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_1_0.txt or copy at
//          http://www.boost.org/LICENSE_1_0.txt)

//! A real-time hierarchical profiler.
//!
//! # What is hierarchical profiling?
//!
//! Hierarchical profiling is based on the observation that games are typically
//! organized into a "tree" of behavior. You have an AI system that does path
//! planning, making tactical decisions, etc. You have a physics system that does
//! collision detection, rigid body dynamics, etc. A tree might look like:
//!
//! - Physics
//!     - Collision detection
//!         - Broad phase
//!         - Narrow phase
//!     - Fluid simulation
//!     - Rigid body simulation
//!         - Collision resolution
//!         - Update positions
//! - AI
//!     - Path planning
//!     - Combat tactics
//!     - Build queue maintenance
//! - Render
//!     - Frustum culling
//!     - Draw call sorting
//!     - Draw call submission
//!     - GPU wait
//!
//! A hierarchical profiler will annotate this tree with how much time each step
//! took. This is an extension of timer-based profiling, where a timer is used to
//! measure how long a block of code takes to execute. Rather than coding up a
//! one-time timer, you merely call `Profiler::enter("description of thing")` and
//! a new entry will be made in the profile tree.
//!
//! The idea came from a 2002 article in Game Programming Gems 3, "Real-Time
//! Hierarchical Profiling" by Greg Hjelstrom and Byon Garrabrant from Westwood
//! Studios. They report having thousands of profile nodes active at a time.
//!
//! There are two major ways to use this library: with explicit profilers, and with an implicit
//! profiler.
//!
//! # Implicit (thread-local) profiler
//!
//! To use the implicit profiler, call `hprof::start_frame()`, `hprof::end_frame()`, and
//! `hprof::enter("name")`. Destructors will take care of the rest. You can access the profiler
//! using `hprof::profiler()`.
//!
//! # Explicit profilers
//!
//! Use `Profiler::new()` and pass it around/store it somewhere (for example, using
//! [`current`](https://github.com/PistonDevelopers/current)).

#[macro_use]
extern crate log;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;

thread_local!(static HPROF: Profiler = Profiler::new("root profiler"));

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
        if curr.ret() == true {
            if let Some(parent) = curr.parent.clone() {
                *curr = parent;
            }
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
    pub start_time: Cell<Instant>,
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
            start_time: Cell::new(Instant::now()),
            recursion: Cell::new(0),
            parent: parent,
            children: RefCell::new(Vec::new())
        }
    }

    /// Reset this node and its children, seting relevant fields to 0.
    pub fn reset(&self) {
        self.calls.set(0);
        self.total_time.set(0);
        self.start_time.set(Instant::now());
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
            self.start_time.set(Instant::now());
        }
        self.recursion.set(rec + 1);
    }

    /// Return from this profile node, returning true if there are no pending recursive calls.
    pub fn ret(&self) -> bool {
        let rec = self.recursion.get();
        if rec == 1 {
            let elapsed = self.start_time.get().elapsed();
            let durr = elapsed.as_secs() * 1000_000_000 + elapsed.subsec_nanos() as u64;
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
        let parent_time = self.parent
                              .as_ref()
                              .map(|p| p.total_time.get())
                              .unwrap_or(self.total_time.get()) as f64;
        let percent = 100.0 * (self.total_time.get() as f64 / parent_time);
        if percent.is_infinite() {
            println!("{name} - {calls} * {each} = {total} @ {hz:.1}hz",
                name  = self.name,
                calls = self.calls.get(),
                each = Nanoseconds((self.total_time.get() as f64 / self.calls.get() as f64) as u64),
                total = Nanoseconds(self.total_time.get()),
                hz = self.calls.get() as f64 / self.total_time.get() as f64 * 1e9f64
            );
        } else {
            println!("{name} - {calls} * {each} = {total} ({percent:.1}%)",
                name  = self.name,
                calls = self.calls.get(),
                each = Nanoseconds((self.total_time.get() as f64 / self.calls.get() as f64) as u64),
                total = Nanoseconds(self.total_time.get()),
                percent = percent
            );
        }
        for c in &*self.children.borrow() {
            c.print(indent+2);
        }
    }
}

pub fn profiler() -> &'static Profiler {
    HPROF.with(|p| unsafe { std::mem::transmute(p) } )
}

pub fn enter(name: &'static str) -> ProfileGuard<'static> {
    HPROF.with(|p| unsafe { std::mem::transmute::<_, &'static Profiler>(p) }.enter(name) )
}

pub fn start_frame() {
    HPROF.with(|p| p.start_frame())
}

pub fn end_frame() {
    HPROF.with(|p| p.end_frame())
}

// used to do a pretty printing of time
struct Nanoseconds(u64);

impl std::fmt::Display for Nanoseconds {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.0 < 1_000 {
            write!(f, "{}ns", self.0)
        } else if self.0 < 1_000_000 {
            write!(f, "{:.1}us", self.0 as f64 / 1_000.)
        } else if self.0 < 1_000_000_000 {
            write!(f, "{:.1}ms", self.0 as f64 / 1_000_000.)
        } else {
            write!(f, "{:.1}s", self.0 as f64 / 1_000_000_000.)
        }
    }
}
