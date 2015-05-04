extern crate hprof;

fn main() {
    loop {
        hprof::start_frame();

        {
            let _g = hprof::enter("setup");
            std::thread::sleep_ms(1);
        }
        {
            let _g = hprof::enter("physics");

            let _g = hprof::enter("collision");
            std::thread::sleep_ms(1);
            drop(_g);

            let _g = hprof::enter("update positions");
            std::thread::sleep_ms(1);
            drop(_g);
        }
        {
            let _g = hprof::enter("render");

            let _g = hprof::enter("cull");
            std::thread::sleep_ms(1);
            drop(_g);

            let _g = hprof::enter("gpu submit");
            std::thread::sleep_ms(2);
            drop(_g);

            let _g = hprof::enter("gpu wait");
            std::thread::sleep_ms(10);
            drop(_g);
        }

        hprof::end_frame();

        // this would usually depend on a debug flag, or use custom functionality for drawing the
        // debug information.
        if true {
            hprof::profiler().print_timing();
        }
        break;
    }
}
