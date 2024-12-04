// TODO: Add trait for Logger and Progressbar -> CLI and TUI / GUI

use std::sync::Arc;

use crate::{
    api_log, api_types, display,
    error::{CubeProgrammerApiError, CubeProgrammerApiResult},
    utility,
};
use derive_more::Into;
use log::debug;
use stm32cubeprogrammer_sys::{libloading, CubeProgrammer_API};

use bon::bon;

pub struct NotConnected;
pub struct Connected;

pub struct CubeProgrammer<T> {
    api: stm32cubeprogrammer_sys::CubeProgrammer_API,

    _phantom: std::marker::PhantomData<T>,
}

#[bon]
impl CubeProgrammer<NotConnected> {
    /// Create new instance of CubeProgrammer
    #[builder]
    pub fn new(
        cube_programmer_dir: impl AsRef<std::path::Path>,
        log_verbosity: Option<api_log::Verbosity>,
        display_callback: Option<Arc<dyn crate::DisplayCallback>>,
    ) -> Result<Self, CubeProgrammerApiError> {
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

        let library =
            Self::load_library(&api_path).map_err(CubeProgrammerApiError::LibLoadingError)?;

        let api = unsafe {
            CubeProgrammer_API::from_library(library)
                .map_err(CubeProgrammerApiError::LibLoadingError)?
        };

        unsafe {
            api.setLoadersPath(utility::path_to_cstring(loader_path).as_ptr());

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

        Ok(Self {
            api,
            _phantom: std::marker::PhantomData::<NotConnected>,
        })
    }

    /// Connect to target via ST-Link probe
    pub fn connect_to_target(
        self,
        st_link: api_types::StLink,
    ) -> CubeProgrammerApiResult<CubeProgrammer<Connected>> {
        let connection_param = st_link.0;
        api_types::ReturnCode::from(unsafe { self.api.connectStLink(connection_param) }).check()?;

        Ok(CubeProgrammer::<Connected> {
            api: self.api,
            _phantom: std::marker::PhantomData::<Connected>,
        })
    }

    /// List all connected ST-Link probes
    pub fn list_connected_st_link_probes(&self) -> Vec<api_types::StLink> {
        let mut debug_parameters =
            std::ptr::null_mut::<stm32cubeprogrammer_sys::debugConnectParameters>();
        let return_value = unsafe { self.api.getStLinkList(&mut debug_parameters, 0) };

        if return_value < 0 || debug_parameters.is_null() {
            return vec![];
        }

        let slice = unsafe { std::slice::from_raw_parts(debug_parameters, return_value as _) };

        let st_links = slice
            .iter()
            .map(|debug_parameters| api_types::StLink(*debug_parameters))
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
                unsafe { libloading::os::windows::Library::new(path)?.into() };

            Ok(library)
        }

        unsafe { load_inner(api_library_path.as_ref()) }
    }
}

impl CubeProgrammer<Connected> {
    /// Disconnect from target
    pub fn disconnect(&self) {
        unsafe { self.api.disconnect() };
    }

    /// Get general device information
    pub fn get_general_device_information(
        &self,
    ) -> CubeProgrammerApiResult<api_types::TargetInformation> {
        let general_information = unsafe { self.api.getDeviceGeneralInf() };

        if general_information.is_null() {
            return Err(CubeProgrammerApiError::CommandReturnNull);
        }

        let general_information = api_types::TargetInformation(unsafe { *general_information });
        Ok(general_information)
    }

    /// Reset target
    pub fn reset_target(&self, reset_mode: api_types::ResetMode) -> CubeProgrammerApiResult<()> {
        api_types::ReturnCode::from(unsafe { self.api.reset(reset_mode.into()) }).check()
    }

    /// Download hex file to target
    pub fn download_hex_file(
        &self,
        file_path: impl AsRef<std::path::Path>,
        skip_erase: bool,
        verify: bool,
    ) -> CubeProgrammerApiResult<()> {
        let file_path = utility::path_to_wide_cstring(file_path);

        api_types::ReturnCode::from(unsafe {
            self.api.downloadFile(
                file_path.as_ptr(),
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
    ) -> CubeProgrammerApiResult<()> {
        let file_path = utility::path_to_wide_cstring(file_path);

        api_types::ReturnCode::from(unsafe {
            self.api.downloadFile(
                file_path.as_ptr(),
                start_address,
                if skip_erase { 1 } else { 0 },
                if verify { 1 } else { 0 },
                std::ptr::null(),
            )
        })
        .check()
    }

    /// Perform mass erase
    pub fn mass_erase(&self) -> CubeProgrammerApiResult<()> {
        api_types::ReturnCode::from(unsafe { self.api.massErase(std::ptr::null_mut()) }).check()
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
    ) -> CubeProgrammerApiResult<()> {
        let return_value = unsafe {
            self.api.firmwareUpgrade(
                utility::path_to_wide_cstring(file_path).as_ptr(),
                start_address,
                if first_install { 1 } else { 0 },
                if verify { 1 } else { 0 },
                if start_stack_after_update { 1 } else { 0 },
            )
        };

        // The command returns 1 on success
        if return_value == 1 {
            Ok(())
        } else {
            Err(CubeProgrammerApiError::CommandReturnCode {
                return_code: return_value,
            })
        }
    }

    /// Save memory to file
    /// Attention: The file path must end with .hex or .bin
    pub fn save_memory_file(
        &self,
        file_path: impl AsRef<std::path::Path>,
        start_address: u32,
        size_bytes: u32,
    ) -> CubeProgrammerApiResult<()> {
        api_types::ReturnCode::from(unsafe {
            self.api.saveMemoryToFile(
                start_address as i32, // TODO: Handle properly
                size_bytes as i32,    // TODO: Handle properly
                utility::path_to_wide_cstring(file_path).as_ptr(),
            )
        })
        .check()
    }
}
