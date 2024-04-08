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
		"Running command: \"{} {}\" in directory: {:?}",
		program,
		args.join(" "),
		current_dir(),
	);
	let ret = Command::new(program).args(args).status();

	match ret.map(|status| (status.success(), status.code())) {
		Ok((true, _)) => println!("updating submodules exited successfully"),
		Ok((false, Some(exit_code))) => panic!("updating submodules failed with error code {}", exit_code),
		Ok((false, None)) => {
			panic!("updating submodules exited with no error code, possibly killed by system, exiting.")
		},
		Err(e) => panic!("updating submodules failed with error: {}", e),
	}
}

fn check_compiler(compiler: &'static str) -> &'static str {
	let args = "-v";

	println!("checking if compiler {} exists", compiler);

	let ret = Command::new(compiler).arg(args).status();

	match ret.map(|status| (status.success(), status.code())) {
		Ok((true, _)) => println!("compiler check exited successfully"),
		Ok((false, Some(exit_code))) => panic!("compiler check failed with error code {}", exit_code),
		Ok((false, None)) => panic!("compiler check exited with no error code, possibly killed by system"),
		Err(e) => panic!("compiler check failed with error: {}", e),
	}
	compiler
}

fn main() {
	#[cfg(all(feature = "gcc", feature = "clang"))]
	compile_error!("gcc OR clang must be enabled, not both.");

	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=src/hardened_malloc/");
	println!("cargo:rerun-if-changed=src/hardened_malloc/.git");
	println!("cargo:rerun-if-changed=src/hardened_malloc/.git/HEAD");
	println!("cargo:rerun-if-changed=src/hardened_malloc/.git/index");
	println!("cargo:rerun-if-changed=src/hardened_malloc/.git/refs/tags");

	let out_dir = env::var("OUT_DIR").unwrap();

	if !Path::new("src/hardened_malloc/Makefile").exists() {
		println!("src/hardened_malloc/Makefile does not exist, running submodule sync");
		update_submodules();
	}

	let compiler = if cfg!(feature = "gcc") {
		check_compiler("gcc")
	} else {
		check_compiler("clang")
	};

	let variant = if cfg!(feature = "light") {
		"light"
	} else {
		"default" // "default" is hardened_malloc's default.mk. this crate's feature
		  // uses "standard" for "default"
	};

	let build_args = [
		format!("VARIANT={}", variant),
		format!("V={}", "1"),
		format!("OUT={}", &out_dir),
		format!("CC={}", compiler),
	];

	//TODO: handle support for explicit make flags like N_ARENA=1 and such
	let mut make_command = Command::new("make");

	println!("running {:?} with args {:?}", make_command, build_args);

	let make_output = make_command
		.current_dir("src/hardened_malloc/")
		.args(build_args)
		.output()
		.unwrap_or_else(|error| {
			panic!("failed to run 'make {}': ", error);
		});

	if !make_output.status.success() {
		panic!(
			"building hardened_malloc failed:\n{:?}\n{}\n{}",
			make_command,
			String::from_utf8_lossy(&make_output.stdout),
			String::from_utf8_lossy(&make_output.stderr)
		);
	}

	if cfg!(feature = "light") {
		println!("cargo:rustc-link-lib=dylib=hardened_malloc-light");
		println!("cargo:rustc-link-search={}", out_dir);
	} else {
		println!("cargo:rustc-link-lib=dylib=hardened_malloc");
		println!("cargo:rustc-link-search={}", out_dir);
	}
}
