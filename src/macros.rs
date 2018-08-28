macro_rules! measure_performance {
    ($function:expr, $variable:expr) => ({
        if cfg!(feature = "performance_test") {
            use std::time::Instant;
            let time_stamp = Instant::now();
            let output = $function;
            $variable = time_stamp.elapsed();
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

macro_rules! run_command {
    ($dir:expr; $command:expr, $( $args:expr),*) => ({
        {
            use std::process::Command;
            Command::new($command)
                .current_dir($dir)
                .args(&[$( $args ),*])
                .output()
                .expect(&format!("failed to execute process: {}", $command))
        }
    });

    ($command:expr, $( $args:expr),*) => ({
        {
            use std::process::Command;
            Command::new($command)
                .args(&[$( $args ),*])
                .output()
                .expect(&format!("failed to execute process: {}", $command))
        }
    });
}

macro_rules! compile_cpp {
    ($( $args:expr),*) => ({
        let output = run_command!("g++", $( $args ),*);

        //print the error if we have one
        let error = String::from_utf8(output.stderr).unwrap();
        if error != "".to_owned() {
            panic!("C++ compile error: \n{}", error);
        }
    });
}