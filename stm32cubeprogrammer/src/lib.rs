pub mod api;

#[cfg(test)]
mod tests {
    use log::info;

    /// Environment variable name for the path to the STM32CubeProgrammer directory
    /// Needs to be the root path of the STM32CubeProgrammer installatios
    /// Example: C:\Program Files\STMicroelectronics\STM32Cube\STM32CubeProgrammer
    const ENV_CUBE_PROGRAMMER_DIR: &str = "STM32_CUBE_PROGRAMMER_DIR";

    use super::*;

    /// Get the path to the STM32CubeProgrammer directory from the environment file
    fn get_path_from_env_file() -> std::path::PathBuf {
        dotenvy::dotenv().unwrap();
        std::env::var(ENV_CUBE_PROGRAMMER_DIR).unwrap().into()
    }

    #[test_log::test]
    fn load_api() {
        dotenvy::dotenv().unwrap();
        
        let api = api::CubeProgrammerApi::new(get_path_from_env_file()).unwrap();
        let probes = api.list_connected_st_link_probes();

        dbg!(probes);
    }
}
