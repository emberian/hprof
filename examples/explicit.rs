extern crate hprof;

fn main() {
    let p = hprof::Profiler::new("main loop");

    loop {
        p.start_frame();

        {
            let _g = p.enter("setup");
            std::thread::sleep_ms(1);
        }
        {
            let _g = p.enter("physics");

            let _g = p.enter("collision");
            std::thread::sleep_ms(1);
            drop(_g);

            let _g = p.enter("update positions");
            std::thread::sleep_ms(1);
            drop(_g);
        }
        {
            let _g = p.enter("render");

            let _g = p.enter("cull");
            std::thread::sleep_ms(1);
            drop(_g);

            let _g = p.enter("gpu submit");
            std::thread::sleep_ms(2);
            drop(_g);

            let _g = p.enter("gpu wait");
            std::thread::sleep_ms(10);
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
