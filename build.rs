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

fn check_compiler_and_linker(compiler: &'static str, linker: &'static str) -> (&'static str, &'static str) {
	println!("checking if compiler {compiler} exists");

	let compiler_ret = Command::new(compiler).arg("--version").status();

	match compiler_ret.map(|status| (status.success(), status.code())) {
		Ok((true, _)) => println!("compiler check exited successfully"),
		Ok((false, Some(exit_code))) => panic!("compiler check failed with error code {exit_code}"),
		Ok((false, None)) => panic!("compiler check exited with no error code, possibly killed by system"),
		Err(e) => panic!("compiler check failed with error: {e}"),
	}

	println!("checking if linker {linker} exists");

	let linker_ret = Command::new(linker).arg("--version").status();

	match linker_ret.map(|status| (status.success(), status.code())) {
		Ok((true, _)) => println!("linker check exited successfully"),
		Ok((false, Some(exit_code))) => {
			if exit_code == 1 {
				println!("linker check exited with exit code 1, assuming it's available");
			}
		},
		Ok((false, None)) => panic!("linker check exited with no error code, possibly killed by system"),
		Err(e) => panic!("linker check failed with error: {e}"),
	}

	(compiler, linker)
}

fn main() {
	#[cfg(all(
		not(feature = "gcc"),
		not(feature = "clang"),
		not(feature = "static"),
		not(feature = "dynamic"),
		not(feature = "light"),
		not(feature = "standard")
	))]
	compile_error!("At least one of each category of feature must be enabled.");

	#[cfg(all(feature = "gcc", feature = "clang"))]
	compile_error!("gcc OR clang compiler must be enabled, not both.");

	#[cfg(all(feature = "static", feature = "dynamic"))]
	compile_error!("static OR dynamic linking must be enabled, not both.");

	#[cfg(all(feature = "light", feature = "standard"))]
	compile_error!("light OR standard variant must be enabled, not both.");

	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=src/hardened_malloc/");
	println!("cargo:rerun-if-changed=src/hardened_malloc/LICENSE");
	println!("cargo:rerun-if-changed=src/hardened_malloc/.git");
	println!("cargo:rerun-if-changed=src/hardened_malloc/.git/HEAD");
	println!("cargo:rerun-if-changed=src/hardened_malloc/.git/index");
	println!("cargo:rerun-if-changed=src/hardened_malloc/.git/refs/tags");

	let out_dir = env::var("OUT_DIR").unwrap();

	if !Path::new("src/hardened_malloc/Makefile").exists() {
		println!("src/hardened_malloc/Makefile does not exist, running submodule sync");
		update_submodules();
	}

	let (compiler, _linker) = if cfg!(feature = "gcc") {
		check_compiler_and_linker("gcc", "ld")
	} else {
		check_compiler_and_linker("clang", "ld")
	};

	// "default" is hardened_malloc's default.mk. this crate's feature uses
	// "standard" for "default"
	let variant = if cfg!(feature = "light") {
		"light"
	} else {
		"default"
	};

	let build_args: Vec<String> = if cfg!(features = "clang") && cfg!(features = "static") {
		vec![
			format!("VARIANT={}", variant),
			format!("CONFIG_STATIC=true"), // only intended to be used by clang
			format!("V={}", "1"),          // verbose (?)
			format!("OUT={}", &out_dir),
			format!("CC={}", compiler),
		]
	} else {
		vec![
			format!("VARIANT={}", variant),
			format!("V={}", "1"), // verbose (?)
			format!("OUT={}", &out_dir),
			format!("CC={}", compiler),
		]
	};

	// TODO: handle support for explicit make flags like N_ARENA=1 and such (should
	// this be crate features on top of the existing variant features/configs?)
	let mut make_command = Command::new("make");

	println!("running {:?} with args {:?}", make_command, build_args);

	let make_output = make_command
		.current_dir("src/hardened_malloc/")
		.args(build_args.clone())
		.output()
		.unwrap_or_else(|error| {
			panic!("failed to run 'make {build_args:?}': {error}");
		});

	if !make_output.status.success() {
		panic!(
			"building hardened_malloc failed:\n{:?}\n{}\n{}",
			make_command,
			String::from_utf8_lossy(&make_output.stdout),
			String::from_utf8_lossy(&make_output.stderr)
		);
	}

	if cfg!(feature = "static") {
		let ar_lib_output = if cfg!(feature = "light") {
			out_dir.clone() + "/libhardened_malloc-light.a"
		} else {
			out_dir.clone() + "/libhardened_malloc.a"
		};

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
			out_dir.clone() + "/new.o",
		];

		let mut ar_command = Command::new("ar");

		println!("running {:?} with args {:?}", ar_command, ar_args);

		let ar_output = ar_command.args(ar_args).output().unwrap_or_else(|error| {
			panic!("Failed to run '{ar_command:?}': {error}");
		});

		if !ar_output.status.success() {
			panic!(
				"creating static lib of hardened_malloc failed:\n{:?}\n{}\n{}",
				ar_command,
				String::from_utf8_lossy(&ar_output.stdout),
				String::from_utf8_lossy(&ar_output.stderr)
			);
		}

		if cfg!(feature = "light") {
			println!("cargo:rustc-link-search={}", out_dir);
			println!("cargo:rustc-link-lib=static=hardened_malloc-light");
		} else {
			println!("cargo:rustc-link-search={}", out_dir);
			println!("cargo:rustc-link-lib=static=hardened_malloc");
		}
	} else if cfg!(feature = "dynamic") {
		// TODO: is this needed?
		let target = env::var("TARGET").unwrap();
		if target.contains("apple") || target.contains("freebsd") || target.contains("openbsd") {
			println!("cargo:rustc-link-lib=dylib=c++");
		} else if target.contains("linux") {
			println!("cargo:rustc-link-lib=dylib=stdc++");
		}

		if cfg!(feature = "light") {
			println!("cargo:rustc-link-lib=dylib=hardened_malloc-light");
			println!("cargo:rustc-link-search={}", out_dir);
		} else {
			println!("cargo:rustc-link-lib=dylib=hardened_malloc");
			println!("cargo:rustc-link-search={}", out_dir);
		}
	}

	println!("cargo:out_dir={}", out_dir);
}
