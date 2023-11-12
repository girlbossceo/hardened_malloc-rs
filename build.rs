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
        Ok((true, _)) => (),
        Ok((false, Some(c))) => panic!(
            "[hardened_malloc-sys]: Command failed with error code {}",
            c
        ),
        Ok((false, None)) => panic!(
            "[hardened_malloc-sys]: Command exited with no error code, possibly killed by system"
        ),
        Err(e) => panic!("[hardened_malloc-sys]: Command failed with error: {}", e),
    }
}

fn main() {
    if !Path::new("src/hardened_malloc/Makefile").exists() {
        println!("src/hardened_malloc/Makefile does not exist, running submodule sync");
        update_submodules();
    }
    let variant;

    if cfg!(feature = "light") {
        variant = "light";
    } else {
        variant = "default"; // "default" is hardened_malloc's default.mk. this crate's feature uses "standard" for "default"
    }

    let build_args = ["VARIANT=".to_owned() + variant, "V=".to_owned() + "1"];

    //TODO: handle support for explicit make flags like N_ARENA=1 and such

    let mut make_command = Command::new("make");
    println!("running {:?} with args {:?}", make_command, build_args);
    let make_output = make_command
        .current_dir("src/hardened_malloc/")
        .args(build_args)
        .output()
        .unwrap_or_else(|error| {
            panic!("Failed to run 'make {}': ", error);
        });
    if !make_output.status.success() {
        panic!(
            "[hardened_malloc-sys]: building hardened_malloc failed:\n{:?}\n{}\n{}",
            make_command,
            String::from_utf8_lossy(&make_output.stdout),
            String::from_utf8_lossy(&make_output.stderr)
        );
    }

    //println!("cargo:rustc-link-search=native=src/hardened_malloc");

    //println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/hardened_malloc/");
    //println!("cargo:out_dir={}", env::var("OUT_DIR").unwrap());
}
