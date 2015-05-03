# `hprof`, a real-time hierarchical profiler

[![Travis](https://img.shields.io/travis/cmr/hprof.svg?style=flat-square)](https://travis-ci.org/cmr/hprof)
[![Crates.io](https://img.shields.io/crates/v/hprof.svg?style=flat-square)](https://crates.io/crates/hprof)

[Documentation](https://cmr.github.io/hprof)

`hprof` is suitable only for getting rough measurements of "systems", rather
than fine-tuned profilng data. Consider using `perf`, `SystemTap`, `DTrace`,
`VTune`, etc for more detailed profiling.

# License

This software is licensed under the [Boost Software
License](http://www.boost.org/users/license.html). In short, you are free to
use, modify, and redistribute in any form without attribution.

# Example

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
