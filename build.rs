#![allow(unused_macros)]
#[macro_use]
mod src;

fn main () {
    let mut current_dir = String::from_utf8(run_command!("pwd").stdout).unwrap();
    current_dir.pop();

    install_sdsl();
    //clear static lib
    run_command!("target"; "rm", "libinterface.a");

    //compile the interface
    compile_cpp!("-std=c++11", "-O3", "-DNDEBUG", "-I", "target/sdsl/sdsl_install/include", "-fpic", "-c", "src_cpp/sdsl_interface.cpp", "-o", "target/sdsl_interface.o");

    //build a static library
    run_command!("target"; "ar", "crus", "libsdsl_interface.a", "sdsl_interface.o");

    println!("cargo:rustc-flags= -L {}/target/sdsl/sdsl_install/lib -l sdsl -l divsufsort -l divsufsort64 -l stdc++", current_dir);
    println!("cargo:rustc-link-search=native=target");
}

fn install_sdsl() {
    run_command!("mkdir", "target");
    let status = run_command!("target"; "mkdir", "sdsl").status;

    //if we create the directory so we have to download the lib
    if status.success() {
        let output = run_command!("target/sdsl"; "git", "clone", "https://github.com/simongog/sdsl-lite.git");
        if !output.status.success() {
            panic!("Error: Could not clone sdsl: \n{}", String::from_utf8(output.stderr).unwrap());
        }

        run_command!("target/sdsl/sdsl-lite"; "rm", "-r", "build");

        let output = run_command!("target/sdsl/sdsl-lite"; "mkdir", "build");
        if !output.status.success() {
            panic!("Error: Could not make build directory: \n{}", String::from_utf8(output.stderr).unwrap());
        }

        let output = run_command!("target/sdsl/sdsl-lite/build"; "cmake", r#"-DCMAKE_CXX_FLAGS="-fpic""#, r#"-DCMAKE_BUILD_TYPE="Release""#, r#"-DCMAKE_INSTALL_PREFIX=../../sdsl_install"#, "..");
        if !output.status.success() {
            panic!("Error: Could not set install parameter sdsl: \n{}", String::from_utf8(output.stderr).unwrap());
        }

        let output = run_command!("target/sdsl/sdsl-lite/build"; "make", "-j", "install");
        if !output.status.success() {
            panic!("Error: Could not install sdsl: \n{}", String::from_utf8(output.stderr).unwrap());
        }
    }
}