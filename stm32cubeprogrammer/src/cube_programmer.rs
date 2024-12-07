// TODO: Add trait for Logger and Progressbar -> CLI and TUI / GUI

use std::sync::{Arc, Mutex};

use crate::{
    api_log, api_types, display,
    error::{CubeProgrammerError, CubeProgrammerResult},
    utility,
};
use derive_more::Into;
use log::debug;
use stm32cubeprogrammer_sys::{libloading, CubeProgrammer_API};

use bon::bon;
/// CubeProgrammer struct which holds the FFI API and provides a rust wrapper around it
pub struct CubeProgrammer {
    api: stm32cubeprogrammer_sys::CubeProgrammer_API,
}

pub struct ConnectedCubeProgrammer {
    programmer: CubeProgrammer,
    connect_parameters: api_types::ConnectParameters,
}

impl ConnectedCubeProgrammer {
    fn api(&self) -> &CubeProgrammer_API {
        &self.programmer.api
    }
}

#[bon]
impl CubeProgrammer {
    /// Create new instance of CubeProgrammer
    #[builder]
    pub fn new(
        cube_programmer_dir: impl AsRef<std::path::Path>,
        log_verbosity: Option<api_log::Verbosity>,
        display_callback: Option<Arc<Mutex<dyn crate::DisplayCallback>>>,
    ) -> Result<Self, CubeProgrammerError> {
        use stm32cubeprogrammer_sys::{PATH_API_LIBRARY_RELATIVE, PATH_LOADER_DIR_RELATIVE};

        let api_path = cube_programmer_dir
            .as_ref()
            .join(PATH_API_LIBRARY_RELATIVE)
            .canonicalize()
            .expect("Failed to get canonical path");

        let loader_path = cube_programmer_dir
            .as_ref()
            .join(PATH_LOADER_DIR_RELATIVE)
            .canonicalize()
            .expect("Failed to get canonical path");

        debug!("API path: {:?}", api_path);
        debug!("Loader path: {:?}", loader_path);
        debug!("Log verbosity: {:?}", log_verbosity);

        if let Some(display_callback) = display_callback {
            debug!("Set display callback handler");
            display::set_display_callback_handler(display_callback);
        }

        let library = Self::load_library(&api_path).map_err(CubeProgrammerError::LibLoading)?;

        let api = unsafe {
            CubeProgrammer_API::from_library(library).map_err(CubeProgrammerError::LibLoading)?
        };

        unsafe {
            api.setLoadersPath(utility::path_to_cstring(loader_path)?.as_ptr());

            {
                let verbosity = log_verbosity.unwrap_or({
                    debug!("Use default verbosity level");
                    api_log::Verbosity::Level3
                });

                debug!("Set verbosity level: {}", verbosity);

                api.setVerbosityLevel(verbosity.into());
            }

            let display_callbacks = stm32cubeprogrammer_sys::displayCallBacks {
                initProgressBar: Some(api_log::display_callback_init_progressbar),
                logMessage: Some(api_log::display_callback_log_message),
                loadBar: Some(api_log::display_callback_load_bar),
            };

            api.setDisplayCallbacks(display_callbacks);
        }

        Ok(Self { api })
    }

    /// Connect to target via ST-Link probe
    pub fn connect_to_target(
        self,
        st_link: &api_types::ConnectParameters,
    ) -> CubeProgrammerResult<ConnectedCubeProgrammer> {
        let connection_param = st_link.0;
        api_types::ReturnCode::<0>::from(unsafe { self.api.connectStLink(connection_param) })
            .check()?;

        Ok(ConnectedCubeProgrammer {
            programmer: self,
            connect_parameters: st_link.clone(),
        })
    }

    /// List all connected ST-Link probes
    pub fn list_connected_st_link_probes(&self) -> Vec<api_types::ConnectParameters> {
        let mut debug_parameters =
            std::ptr::null_mut::<stm32cubeprogrammer_sys::debugConnectParameters>();
        let return_value = unsafe { self.api.getStLinkList(&mut debug_parameters, 0) };

        if return_value < 0 || debug_parameters.is_null() {
            return vec![];
        }

        let slice = unsafe { std::slice::from_raw_parts(debug_parameters, return_value as _) };

        let st_links = slice
            .iter()
            .map(|debug_parameters| api_types::ConnectParameters(*debug_parameters))
            .collect();

        // Free the memory allocated by the API
        unsafe {
            self.api.deleteInterfaceList();
        }

        st_links
    }

    /// Load the dynamic library with libloading
    fn load_library(
        api_library_path: impl AsRef<std::ffi::OsStr>,
    ) -> Result<libloading::Library, libloading::Error> {
        #[cfg(windows)]
        unsafe fn load_inner(
            path: impl AsRef<std::ffi::OsStr>,
        ) -> Result<libloading::Library, libloading::Error> {
            let library: libloading::Library = unsafe {
                libloading::os::windows::Library::load_with_flags(
                    path,
                    libloading::os::windows::LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR
                        | libloading::os::windows::LOAD_LIBRARY_SEARCH_SYSTEM32
                        | libloading::os::windows::LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
                )?
                .into()
            };

            Ok(library)
        }

        #[cfg(unix)]
        unsafe fn load_inner(
            path: impl AsRef<std::ffi::OsStr>,
        ) -> Result<libloading::Library, libloading::Error> {
            let library: libloading::Library =
                unsafe { libloading::os::unix::Library::new(path)?.into() };

            Ok(library)
        }

        unsafe { load_inner(api_library_path.as_ref()) }
    }
}

impl ConnectedCubeProgrammer {
    /// Disconnect from target
    pub fn disconnect(self) -> CubeProgrammer {
        unsafe { self.api().disconnect() };
        self.programmer
    }

    /// Get general device information
    pub fn get_general_device_information(
        &self,
    ) -> CubeProgrammerResult<api_types::TargetInformation> {
        self.check_connection()?;

        let general_information = unsafe { self.api().getDeviceGeneralInf() };

        if general_information.is_null() {
            return Err(CubeProgrammerError::CommandReturnNull);
        }

        let general_information = api_types::TargetInformation(unsafe { *general_information });
        Ok(general_information)
    }

    /// Reset target
    pub fn reset_target(&self, reset_mode: api_types::ResetMode) -> CubeProgrammerResult<()> {
        self.check_connection()?;

        api_types::ReturnCode::<0>::from(unsafe { self.api().reset(reset_mode.into()) }).check()
    }

    /// Download hex file to target
    pub fn download_hex_file(
        &self,
        file_path: impl AsRef<std::path::Path>,
        skip_erase: bool,
        verify: bool,
    ) -> CubeProgrammerResult<()> {
        // Validate if the given file is a valid hex file if the feature is enabled
        #[cfg(feature = "ihex")]
        {
            // Check if the given file is really a hex file
            // Unfortunately, the CubeProgrammer API does not check this and simply programs to address 0 if a bin file is passed
            let file_content = std::fs::read(&file_path).map_err(CubeProgrammerError::FileIo)?;
            let file_content =
                std::str::from_utf8(&file_content).map_err(|_| CubeProgrammerError::Parameter {
                    message: "Invalid intelhex file".to_string(),
                })?;

            let reader = ihex::Reader::new_with_options(
                &file_content,
                ihex::ReaderOptions {
                    stop_after_first_error: true,
                    stop_after_eof: true,
                },
            );

            for record in reader {
                match record {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(CubeProgrammerError::Parameter {
                            message: format!("Invalid intelhex file: {}", e),
                        });
                    }
                }
            }
        }

        self.check_connection()?;

        let file_path = utility::path_to_widestring(file_path);

        api_types::ReturnCode::<0>::from(unsafe {
            self.api().downloadFile(
                file_path?.as_ptr(),
                0,
                if skip_erase { 1 } else { 0 },
                if verify { 1 } else { 0 },
                std::ptr::null(),
            )
        })
        .check()
    }

    /// Download binary file to target
    pub fn download_bin_file(
        &self,
        file_path: impl AsRef<std::path::Path>,
        start_address: u32,
        skip_erase: bool,
        verify: bool,
    ) -> CubeProgrammerResult<()> {
        self.check_connection()?;

        let file_path = utility::path_to_widestring(file_path);

        api_types::ReturnCode::<0>::from(unsafe {
            self.api().downloadFile(
                file_path?.as_ptr(),
                start_address,
                if skip_erase { 1 } else { 0 },
                if verify { 1 } else { 0 },
                std::ptr::null(),
            )
        })
        .check()
    }

    /// Perform mass erase
    pub fn mass_erase(&self) -> CubeProgrammerResult<()> {
        self.check_connection()?;

        api_types::ReturnCode::<0>::from(unsafe { self.api().massErase(std::ptr::null_mut()) })
            .check()
    }

    /// Update BLE stack
    /// TODO: Check for stm32wb
    pub fn update_ble_stack(
        &self,
        file_path: impl AsRef<std::path::Path>,
        start_address: u32,
        first_install: bool,
        verify: bool,
        start_stack_after_update: bool,
    ) -> CubeProgrammerResult<()> {
        self.check_connection()?;

        api_types::ReturnCode::<1>::from(unsafe {
            self.api().firmwareUpgrade(
                utility::path_to_widestring(file_path)?.as_ptr(),
                start_address,
                if first_install { 1 } else { 0 },
                if verify { 1 } else { 0 },
                if start_stack_after_update { 1 } else { 0 },
            )
        })
        .check()
    }

    /// Save memory to file
    /// Attention: The file path must end with .hex or .bin
    pub fn save_memory_file(
        &self,
        file_path: impl AsRef<std::path::Path>,
        start_address: u32,
        size_bytes: u32,
    ) -> CubeProgrammerResult<()> {
        self.check_connection()?;

        api_types::ReturnCode::<0>::from(unsafe {
            self.api().saveMemoryToFile(
                i32::try_from(start_address).map_err(|x| CubeProgrammerError::Parameter {
                    message: format!("Start address exceeds max value: {}", x),
                })?,
                i32::try_from(size_bytes).map_err(|x| CubeProgrammerError::Parameter {
                    message: format!("Size exceeds max value: {}", x),
                })?,
                utility::path_to_widestring(file_path)?.as_ptr(),
            )
        })
        .check()
    }

    /// Enable roud out protection level 1 (0xBB)
    pub fn enable_read_out_protection(&self) -> CubeProgrammerResult<()> {
        /// Command according to Example 3 of the CubeProgrammer API documentation
        const COMMAND_ENABLE_ROP_LEVEL_1: &str = "-ob rdp=0xbb";

        self.check_connection()?;

        api_types::ReturnCode::<0>::from(unsafe {
            self.api().sendOptionBytesCmd(
                utility::string_to_cstring(COMMAND_ENABLE_ROP_LEVEL_1)?.as_ptr()
                    as *mut std::ffi::c_char,
            )
        })
        .check()
    }

    /// Disable read out protection
    /// Attention: This command will erase the device memory
    pub fn disable_read_out_protection(&self) -> CubeProgrammerResult<()> {
        self.check_connection()?;
        api_types::ReturnCode::<0>::from(unsafe { self.api().readUnprotect() }).check()?;
        Ok(())
    }

    /// Check connection to target
    /// Consumes self and and only returns self if the connection is still maintained
    /// If the connection is lost, the user is forced to reconnect
    fn check_connection(&self) -> CubeProgrammerResult<()> {
        api_types::ReturnCode::<1>::from(unsafe { self.api().checkDeviceConnection() })
            .check()
            .map_err(|_| CubeProgrammerError::ConnectionLost)
    }

    /// Convenience function to reconnect using the same connect parameters which were passed in [`CubeProgrammer::connect_to_target`]
    /// Should be called if a function returns [`CubeProgrammerError::ConnectionLost`]
    pub fn reconnect(self) -> CubeProgrammerResult<Self> {
        let connect_parameters = self.connect_parameters.clone();
        let programmer = self.disconnect();
        let connected = programmer.connect_to_target(&connect_parameters)?;

        Ok(connected)
    }
}
