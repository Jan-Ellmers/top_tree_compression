macro_rules! measure_performance {
    ($function:expr, $message:expr) => ({
        if cfg!(feature = "performance_test") {
            use std::time::Instant;
            let time_stamp = Instant::now();
            let output = $function;
            println!("{} {:?}", $message, time_stamp.elapsed());
            output
        } else {
            $function
        }
    });
}

macro_rules! debug {
    ($( $args:expr),+) => ({
        if cfg!(feature= "debug") {
            println!( $( $args ),+ );
        }
    });
}