#[macro_use]
extern crate hprof;

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
