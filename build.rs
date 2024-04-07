use std::{
    env::{self, current_dir},
    path::Path,
    process::Command,
};

/// If submodules were not synced, sync them to actually build hardened_malloc
fn update_submodules() {
    let program = "git";
    let args = ["submodule", "update", "--init", "--recursive"];
    println!(
        "[hardened_malloc-sys]: Running command: \"{} {}\" in directory: {:?}",
        program,
        args.join(" "),
        current_dir(),
    );
    let ret = Command::new(program).args(args).status();

    match ret.map(|status| (status.success(), status.code())) {
        Ok((true, _)) => println!("[hardened_malloc-sys]: Updating submodules exited successfully"),
        Ok((false, Some(exit_code))) => panic!(
            "[hardened_malloc-sys]: Updating submodules failed with error code {}",
            exit_code
        ),
        Ok((false, None)) => panic!(
            "[hardened_malloc-sys]: Updating submodules exited with no error code, possibly killed by system, exiting."
        ),
        Err(e) => panic!("[hardened_malloc-sys]: Updating submodules failed with error: {}", e),
    }
}

fn check_compiler(compiler: &str) {
    let args = "-v";

    println!(
        "[hardened_malloc-sys]: Checking if compiler {} exists",
        compiler
    );

    let ret = Command::new(compiler).arg(args).status();

    match ret.map(|status| (status.success(), status.code())) {
        Ok((true, _)) => println!("[hardened_malloc-sys]: Compiler check exited successfully"),
        Ok((false, Some(exit_code))) => panic!(
            "[hardened_malloc-sys]: Compiler check failed with error code {}",
            exit_code
        ),
        Ok((false, None)) => panic!(
            "[hardened_malloc-sys]: Compiler check exited with no error code, possibly killed by system"
        ),
        Err(e) => panic!("[hardened_malloc-sys]: Compiler check failed with error: {}", e),
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/hardened_malloc/");
    println!("cargo:rerun-if-changed=src/hardened_malloc/.git");
    println!("cargo:rerun-if-changed=src/hardened_malloc/.git/HEAD");
    println!("cargo:rerun-if-changed=src/hardened_malloc/.git/index");
    println!("cargo:rerun-if-changed=src/hardened_malloc/.git/refs/tags");

    let out_dir = env::var("OUT_DIR").unwrap();
    let current_working_directory = current_dir().unwrap();

    if !Path::new("src/hardened_malloc/Makefile").exists() {
        println!("src/hardened_malloc/Makefile does not exist, running submodule sync");
        update_submodules();
    }

    let compiler = if cfg!(feature = "gcc") {
        check_compiler("gcc");
        "gcc"
    } else {
        check_compiler("clang");
        "clang"
    };

    let variant = if cfg!(feature = "light") {
        "light"
    } else {
        "default" // "default" is hardened_malloc's default.mk. this crate's feature uses "standard" for "default"
    };

    let build_args = [
        "VARIANT=".to_owned() + variant,
        "V=".to_owned() + "1",
        "OUT=".to_owned() + &out_dir,
        "CC=".to_owned() + compiler,
    ];

    //TODO: handle support for explicit make flags like N_ARENA=1 and such

    let mut make_command = Command::new("make");

    println!("running {:?} with args {:?}", make_command, build_args);

    let make_output = make_command
        .current_dir("src/hardened_malloc/")
        .args(build_args)
        .output()
        .unwrap_or_else(|error| {
            panic!("[hardened_malloc-sys]: Failed to run 'make {}': ", error);
        });

    if !make_output.status.success() {
        panic!(
            "[hardened_malloc-sys]: building hardened_malloc failed:\n{:?}\n{}\n{}",
            make_command,
            String::from_utf8_lossy(&make_output.stdout),
            String::from_utf8_lossy(&make_output.stderr)
        );
    }

    let ar_lib_output = if cfg!(feature = "light") {
        out_dir.clone() + "/libhardened_malloc-light.a"
    } else {
        out_dir.clone() + "/libhardened_malloc.a"
    };

    // TOOD: improve this
    let ar_args = [
        "rcs".to_owned(),
        ar_lib_output,
        out_dir.clone() + "/chacha.o",
        out_dir.clone() + "/h_malloc.o",
        out_dir.clone() + "/memory.o",
        out_dir.clone() + "/new.o",
        out_dir.clone() + "/pages.o",
        out_dir.clone() + "/random.o",
        out_dir.clone() + "/util.o",
    ];

    let mut ar_command = Command::new("ar");

    println!("running {:?} with args {:?}", ar_command, ar_args);

    let ar_output = ar_command
        .args(ar_args)
        .output()
        .unwrap_or_else(|error| {
            panic!("[hardened_malloc-sys]: Failed to run 'ar {}': ", error);
        });

    if !ar_output.status.success() {
        panic!(
            "[hardened_malloc-sys]: creating static lib of hardened_malloc failed:\n{:?}\n{}\n{}",
            ar_command,
            String::from_utf8_lossy(&ar_output.stdout),
            String::from_utf8_lossy(&ar_output.stderr)
        );
    }

    println!(
        "[hardened_malloc-sys]: current working directory: {}",
        current_working_directory.display()
    );

    println!("[hardened_malloc-sys]: OUT_DIR={}", out_dir);

    if cfg!(feature = "light") {
        println!("cargo:rustc-link-lib=static=hardened_malloc-light");
        println!("cargo:rustc-link-search={}", out_dir);
    } else {
        println!("cargo:rustc-link-lib=static=hardened_malloc");
        println!("cargo:rustc-link-search={}", out_dir);
    }
}
