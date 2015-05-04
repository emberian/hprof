extern crate hprof;

fn main() {
    let p = hprof::Profiler::new("main loop");

    loop {
        p.start_frame();

        {
            p.enter_noguard("setup");
            std::thread::sleep_ms(1);
            p.leave();
        }
        {
            p.enter_noguard("physics");

            p.enter_noguard("collision");
            std::thread::sleep_ms(1);
            p.leave();

            p.enter_noguard("update positions");
            std::thread::sleep_ms(1);
            p.leave();

            p.leave();
        }
        {
            p.enter_noguard("render");

            p.enter_noguard("cull");
            std::thread::sleep_ms(1);
            p.leave();

            p.enter_noguard("gpu submit");
            std::thread::sleep_ms(2);
            p.leave();

            p.enter_noguard("gpu wait");
            std::thread::sleep_ms(10);
            p.leave();

            p.leave();
        }

        p.end_frame();

        // this would usually depend on a debug flag, or use custom functionality for drawing the
        // debug information.
        if true {
            p.print_timing();
        }
        break;
    }
}
