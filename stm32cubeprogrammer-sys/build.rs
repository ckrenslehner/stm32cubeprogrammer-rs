use std::path::PathBuf;

const STM32_CUBE_PROGRAMMER_API_PATH: &str = "include/CubeProgrammer_API.h";

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

    let out_path = PathBuf::from("src");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
