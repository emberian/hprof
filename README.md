# `hprof`, a real-time hierarchical profiler

[![Travis](https://img.shields.io/travis/cmr/hprof.svg?style=flat-square)](https://travis-ci.org/cmr/hprof)
[![Crates.io](https://img.shields.io/crates/v/hprof.svg?style=flat-square)](https://crates.io/crates/hprof)

[Documentation](https://cmr.github.io/hprof)

`hprof` is suitable only for getting rough measurements of "systems", rather
than fine-tuned profiling data. Consider using `perf`, `SystemTap`, `DTrace`,
`VTune`, etc for more detailed profiling.

# What is hierarchical profiling?

Hierarchical profiling is based on the observation that games are typically
organized into a "tree" of behavior. You have an AI system that does path
planning, making tactical decisions, etc. You have a physics system that does
collision detection, rigid body dynamics, etc. A tree might look like:

- Physics
    - Collision detection
        - Broad phase
        - Narrow phase
    - Fluid simulation
    - Rigid body simulation
        - Collision resolution
        - Update positions
- AI
    - Path planning
    - Combat tactics
    - Build queue maintenance
- Render
    - Frustum culling
    - Draw call sorting
    - Draw call submission
    - GPU wait

A hierarchical profiler will annotate this tree with how much time each step
took. This is an extension of timer-based profiling, where a timer is used to
measure how long a block of code takes to execute. Rather than coding up a
one-time timer, you merely call `Profiler::enter("description of thing")` and
a new entry will be made in the profile tree.

The idea came from a 2002 article in Game Programming Gems 3, "Real-Time
Hierarchical Profiling" by Greg Hjelstrom and Byon Garrabrant from Westwood
Studios. They report having thousands of profile nodes active at a time.

# License


This software is licensed under the [Boost Software
License](http://www.boost.org/users/license.html). In short, you are free to
use, modify, and redistribute in any form without attribution.

# Unidiomatic Example

```rust
fn main() {
    let p = hprof::Profiler::new("main loop");

    loop {
        p.start_frame();

        p.enter_noguard("setup");
        std::thread::sleep_ms(1);
        p.leave();

        p.enter_noguard("physics");
        std::thread::sleep_ms(2);
        p.leave();

        p.enter_noguard("render");
        std::thread::sleep_ms(8);
        p.leave();

        p.end_frame();

        // this would usually depend on debug data, or use custom functionality for drawing the
        // debug information.
        if true {
            p.print_timing();
        }
        break;
    }
}
```

Output:

```
Timing information for main loop:
  setup - 1149702ns (10.062608%)
  physics - 2116811ns (18.527096%)
  render - 8141904ns (71.260892%)
```

A more typical usage would just call `p.enter("foo")` at the start of a large
chunk of processing that should be measured, and have the guards call `leave`
automatically.
