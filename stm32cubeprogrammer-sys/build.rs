use std::path::PathBuf;

const STM32_CUBE_PROGRAMMER_API_PATH: &str = "include/CubeProgrammer_API.h";

#[cfg(target_os = "windows")]
const BINDINGS_FILE_NAME: &str = "bindings_windows.rs";
#[cfg(target_os = "unix")]
const BINDINGS_FILE_NAME: &str = "bindings_unix.rs";

fn main() {
    let bindings = bindgen::Builder::default()
        .header(STM32_CUBE_PROGRAMMER_API_PATH)
        .clang_arg("-x")
        .clang_arg("c++")
        // Configure bindgen to generate libloading bindings
        .dynamic_library_name("CubeProgrammer_API")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from("src").join(BINDINGS_FILE_NAME);
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}
