use std::{env, process::Command, path::Path};

/// If submodules were not synced, sync them to actually build hardened_malloc
fn update_submodules() {
    let program = "git";
    let dir = "../";
    let args = ["submodule", "update", "--init", "--recursive"];
    println!(
        "[hardened_malloc-sys]: Running command: \"{} {}\" in directory: {}",
        program,
        args.join(" "),
        dir
    );
    let ret = Command::new(program).current_dir(dir).args(args).status();

    match ret.map(|status| (status.success(), status.code())) {
        Ok((true, _)) => (),
        Ok((false, Some(c))) => panic!("[hardened_malloc-sys]: Command failed with error code {}", c),
        Ok((false, None)) => panic!("[hardened_malloc-sys]: Command exited with no error code, possibly killed by system"),
        Err(e) => panic!("[hardened_malloc-sys]: Command failed with error: {}", e),
    }
}

fn main() {
    if !Path::new("src/hardened_malloc/Makefile").exists() {
        update_submodules();
    }
    let variant: &str;
    
    if cfg!(feature = "light") {
        variant = "light";
    } else {
        variant = "default";
    }

    //TODO: handle support for explicit make flags like N_ARENA=1 and such

    let mut make_command = Command::new("make");
    let make_output = make_command
    .current_dir("src/hardened_malloc/")
    .env("V", "1") // always verbose mode for cargo
    .env("VARIANT", variant)
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