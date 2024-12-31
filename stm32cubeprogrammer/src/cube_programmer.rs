use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{
    api_log, api_types, display,
    error::{CubeProgrammerError, CubeProgrammerResult},
    utility,
};

use bon::bon;
use derive_more::Into;
use log::{debug, error};
use stm32cubeprogrammer_sys::SRAM_BASE_ADDRESS;

type ProbeRegistry = HashMap<crate::probe::Serial, Option<crate::probe::Probe>>;

/// Struct which holds the FFI API and helps with loading and setting up the CubeProgrammer API
pub struct CubeProgrammerApi {
    /// API
    api: stm32cubeprogrammer_sys::CubeProgrammer_API,

    /// HashMap to store connected probes. The key is the serial number of the probe
    probe_registry: Rc<RefCell<ProbeRegistry>>,
}

/// ConnectedCubeProgrammer
/// State transitions:
/// - ConnectedCubeProgrammer -> CubeProgrammer
#[derive(Clone)]
pub struct ConnectedCubeProgrammer<'a> {
    api: &'a stm32cubeprogrammer_sys::CubeProgrammer_API,
    probe_registry: Rc<RefCell<ProbeRegistry>>,
    probe: crate::probe::Probe,
    general_information: api_types::TargetInformation,
}

/// ConnectedFusCubeProgrammer.
/// State transitions:
/// - ConnectedFusCubeProgrammer -> CubeProgrammer
#[derive(Clone)]
pub struct ConnectedFusCubeProgrammer<'a> {
    programmer: ConnectedCubeProgrammer<'a>,
    fus_info: crate::fus::Information,
}

#[bon]
impl CubeProgrammerApi {
    /// Create new instance of CubeProgrammerApi
    /// - Load the CubeProgrammer API library
    /// - Set the verbosity level
    /// - Set the display callback handler
    /// - Set the loader path
    #[builder]
    pub fn new(
        cube_programmer_dir: &impl AsRef<std::path::Path>,
        log_verbosity: Option<api_log::Verbosity>,
        display_callback: Option<Arc<Mutex<dyn crate::DisplayCallback>>>,
    ) -> Result<Self, CubeProgrammerError> {
        use stm32cubeprogrammer_sys::{PATH_API_LIBRARY_RELATIVE, PATH_LOADER_DIR_RELATIVE};

        let api_path = cube_programmer_dir
            .as_ref()
            .join(PATH_API_LIBRARY_RELATIVE)
            .canonicalize()
            .map_err(CubeProgrammerError::FileIo)?;

        let loader_path = cube_programmer_dir
            .as_ref()
            .join(PATH_LOADER_DIR_RELATIVE)
            .canonicalize()
            .map_err(CubeProgrammerError::FileIo)?;

        debug!("API path: {:?}", api_path);
        debug!("Loader path: {:?}", loader_path);

        let library = Self::load_library(&api_path).map_err(CubeProgrammerError::LibLoading)?;

        let api = unsafe {
            stm32cubeprogrammer_sys::CubeProgrammer_API::from_library(library)
                .map_err(CubeProgrammerError::LibLoading)?
        };

        if let Some(display_callback) = display_callback {
            debug!("Set display callback handler");
            display::set_display_callback_handler(display_callback);
        }

        unsafe {
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
            api.setLoadersPath(utility::path_to_cstring(loader_path)?.as_ptr());
        }

        Ok(Self {
            api,
            probe_registry: Rc::new(RefCell::new(HashMap::new())),
        })
    }

    fn scan_for_probes(&self) {
        let mut debug_parameters =
            std::ptr::null_mut::<stm32cubeprogrammer_sys::debugConnectParameters>();
        let return_value = unsafe { self.api.getStLinkList(&mut debug_parameters, 0) };

        if return_value < 0 || debug_parameters.is_null() {
            return;
        }

        let slice = unsafe {
            std::slice::from_raw_parts(
                debug_parameters as *mut crate::probe::Probe,
                return_value as _,
            )
        };

        let mut connected_probes = self.probe_registry.borrow_mut();

        // Delete all entries where the value is not None -> There is no active connection
        connected_probes.retain(|_, value| value.is_none());

        for probe in slice {
            // Only insert if the key is not already present
            connected_probes
                .entry(probe.serial_number().to_string().into())
                .or_insert_with(|| Some(probe.clone()));
        }

        // Free the memory allocated by the API
        unsafe {
            self.api.deleteInterfaceList();
        }
    }

    /// List available probes. Scans for probes and then returns the list of probes which are not already in use
    pub fn list_available_probes(&self) -> Vec<crate::probe::Serial> {
        self.scan_for_probes();

        let connected_probes = self.probe_registry.borrow();

        connected_probes
            .values()
            .filter_map(|probe| {
                probe
                    .as_ref()
                    .map(|probe| probe.serial_number().to_string().into())
            })
            .collect()
    }

    /// Connect to a target via a given probe
    pub fn connect_to_target(
        &self,
        probe_serial_number: &crate::probe::Serial,
        protocol: &crate::probe::Protocol,
        connection_parameters: &crate::probe::ConnectionParameters,
    ) -> CubeProgrammerResult<ConnectedCubeProgrammer> {
        let mut connected_probes = self.probe_registry.borrow_mut();

        if let Some(probe) = connected_probes.get_mut(probe_serial_number) {
            if let Some(inner) = probe.take() {
                // Try to connect to the target with the probe
                match api_types::ReturnCode::<0>::from(unsafe {
                    self.api.connectStLink(*crate::probe::Probe::new(
                        &inner,
                        protocol,
                        connection_parameters,
                    ))
                })
                .check(crate::error::Action::Connect)
                {
                    Ok(_) => {
                        // Try to get the general device information
                        let general_information = unsafe { self.api.getDeviceGeneralInf() };
                        if general_information.is_null() {
                            // Reinsert the probe into the connected_probes HashMap
                            *probe = Some(inner);

                            unsafe { self.api.disconnect() };

                            return Err(CubeProgrammerError::ActionOutputUnexpected {
                                action: crate::error::Action::ReadTargetInfo,
                                unexpected_output: crate::error::UnexpectedOutput::Null,
                            });
                        }

                        // We could connect and get the general information
                        let general_information =
                            api_types::TargetInformation(unsafe { *general_information });

                        Ok(ConnectedCubeProgrammer {
                            api: &self.api,
                            probe: inner,
                            probe_registry: Rc::clone(&self.probe_registry),
                            general_information,
                        })
                    }
                    Err(e) => {
                        error!(
                            "Cannot connect to target via probe with serial number: {}",
                            probe_serial_number
                        );

                        // Reinsert the probe into the connected_probes HashMap
                        *probe = Some(inner);

                        Err(e)
                    }
                }
            } else {
                Err(CubeProgrammerError::Parameter {
                    action: crate::error::Action::Connect,
                    message: format!(
                        "Probe with serial number {} already in use",
                        probe_serial_number
                    ),
                })
            }
        } else {
            Err(CubeProgrammerError::Parameter {
                action: crate::error::Action::Connect,
                message: format!("Probe with serial number {} not found", probe_serial_number),
            })
        }
    }

    /// Connect to the firmware update service (FUS) of a target via a given probe
    pub fn connect_to_target_fus(
        &self,
        probe_serial_number: &crate::probe::Serial,
        protocol: &crate::probe::Protocol,
    ) -> CubeProgrammerResult<ConnectedFusCubeProgrammer> {
        // Connect with hardware reset an normal mode
        let connected = self.connect_to_target(
            probe_serial_number,
            protocol,
            &crate::probe::ConnectionParameters {
                frequency: crate::probe::Frequency::Highest,
                reset_mode: crate::probe::ResetMode::Hardware,
                connection_mode: crate::probe::ConnectionMode::Normal,
            },
        )?;

        if !connected.supports_fus() {
            let target_information = connected.target_information().clone();
            connected.disconnect();

            return Err(CubeProgrammerError::NotSupported {
                message: format!(
                    "FUS not supported by connected target series: {}",
                    target_information.series()
                ),
            });
        }

        // Start the FUS
        api_types::ReturnCode::<1>::from(unsafe { connected.api.startFus() })
            .check(crate::error::Action::StartFirmwareUpdateService)?;

        // Disconnect
        connected.disconnect();

        // Reconnect with hot plug
        let connected = self.connect_to_target(
            probe_serial_number,
            protocol,
            &crate::probe::ConnectionParameters {
                frequency: crate::probe::Frequency::Highest,
                reset_mode: crate::probe::ResetMode::Hardware,
                connection_mode: crate::probe::ConnectionMode::HotPlug,
            },
        )?;

        // Read the FUS information
        let fus_info = connected.read_fus_info()?;

        Ok(ConnectedFusCubeProgrammer {
            programmer: connected,
            fus_info,
        })
    }

    /// Load the dynamic library with libloading
    fn load_library(
        api_library_path: impl AsRef<std::ffi::OsStr>,
    ) -> Result<
        stm32cubeprogrammer_sys::libloading::Library,
        stm32cubeprogrammer_sys::libloading::Error,
    > {
        #[cfg(windows)]
        unsafe fn load_inner(
            path: impl AsRef<std::ffi::OsStr>,
        ) -> Result<
            stm32cubeprogrammer_sys::libloading::Library,
            stm32cubeprogrammer_sys::libloading::Error,
        > {
            use stm32cubeprogrammer_sys::libloading;

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

impl std::fmt::Debug for CubeProgrammerApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CubeProgrammerApi").finish_non_exhaustive()
    }
}

impl Drop for ConnectedCubeProgrammer<'_> {
    /// Disconnect and re-insert the probe into the probe registry of the api
    fn drop(&mut self) {
        unsafe {
            self.api.disconnect();
        }

        // Re-insert probe into probe registry
        let mut registry = self.probe_registry.borrow_mut();
        registry.insert(
            self.probe.serial_number().to_owned().into(),
            Some(self.probe.clone()),
        );
    }
}

impl ConnectedCubeProgrammer<'_> {
    /// Disconnect from target
    pub fn disconnect(self) {
        // Consume self -> Drop is called to disconnect
    }

    /// Get general device information
    pub fn target_information(&self) -> &api_types::TargetInformation {
        &self.general_information
    }

    /// Check if the connected target supports the firmware update service (FUS)
    fn supports_fus(&self) -> bool {
        // TODO: Add support for wb1x
        self.general_information.name().eq("STM32WB5x/35xx")
    }

    /// Reads the firmware update service (FUS) information from the shared SRAM2A
    /// To read the device info table the following procedure needs to be followed ([source](https://wiki.st.com/stm32mcu/wiki/Connectivity:STM32WB_FUS)):
    /// - Disconnect
    /// - Connect (mode: normal ; reset: hardware)
    /// - Start FUS
    /// - Disconnect
    /// - Connect (mode: normal ; reset: hot-plug)
    /// - Read the device info table
    ///
    /// The connect/disconnect procedure above needs to be performed before calling this function
    fn read_fus_info(&self) -> CubeProgrammerResult<crate::fus::Information> {
        /// Helper function to convert the version word to major, minor, sub
        fn u32_to_version(version: u32) -> crate::fus::Version {
            const INFO_VERSION_MAJOR_OFFSET: u32 = 24;
            const INFO_VERSION_MAJOR_MASK: u32 = 0xff000000;
            const INFO_VERSION_MINOR_OFFSET: u32 = 16;
            const INFO_VERSION_MINOR_MASK: u32 = 0x00ff0000;
            const INFO_VERSION_SUB_OFFSET: u32 = 8;
            const INFO_VERSION_SUB_MASK: u32 = 0x0000ff00;

            crate::fus::Version {
                major: ((version & INFO_VERSION_MAJOR_MASK) >> INFO_VERSION_MAJOR_OFFSET) as u8,
                minor: ((version & INFO_VERSION_MINOR_MASK) >> INFO_VERSION_MINOR_OFFSET) as u8,
                sub: ((version & INFO_VERSION_SUB_MASK) >> INFO_VERSION_SUB_OFFSET) as u8,
            }
        }

        /// Keyword to check if the FUS device info table is valid
        const FUS_DEVICE_INFO_TABLE_VALIDITY_KEYWORD: u32 = 0xA94656B9;
        /// Offset of the shared RAM
        /// TODO: Add support for WB1
        const SRAM2A_BASE_ADDRESS: u32 = SRAM_BASE_ADDRESS + 0x00030000;

        /// FUS device info table struct
        /// Ported to rust from `STM32_WPAN/interface/patterns/ble_thread/tl/mbox_def.h`
        #[repr(C, packed)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
        struct MBFusDeviceInfoTable {
            /// Needs to be equal to `FUS_DEVICE_INFO_TABLE_VALIDITY_KEYWORD` for the table to be valid
            device_info_table_state: u32,
            reserved1: u8,
            last_fus_active_state: u8,
            last_wireless_stack_state: u8,
            current_wireless_stack_type: u8,
            safe_boot_version: u32,
            fus_version: u32,
            fus_memory_size: u32,
            wireless_stack_version: u32,
            wireless_stack_memory_size: u32,
            wireless_firmware_ble_info: u32,
            wireless_firmware_thread_info: u32,
            reserved2: u32,
            uid64: u64,
            device_id: u16,
        }

        let info_table_address = self.read_memory::<u32>(SRAM2A_BASE_ADDRESS, 1)?[0];

        if info_table_address == 0 {
            return Err(CubeProgrammerError::ActionOutputUnexpected {
                action: crate::error::Action::ReadFirmwareUpdateServiceInfo,
                unexpected_output: crate::error::UnexpectedOutput::Null,
            });
        }

        let info_table = self.read_memory::<MBFusDeviceInfoTable>(info_table_address, 1)?[0];

        if info_table.device_info_table_state != FUS_DEVICE_INFO_TABLE_VALIDITY_KEYWORD {
            error!("Read FUS info table is not valid. Return default FUS info");
            return Err(CubeProgrammerError::ActionOutputUnexpected {
                action: crate::error::Action::ReadFirmwareUpdateServiceInfo,
                unexpected_output: crate::error::UnexpectedOutput::Null,
            });
        }

        Ok(crate::fus::Information {
            fus_version: u32_to_version(info_table.fus_version),
            wireless_stack_version: u32_to_version(info_table.wireless_stack_version),
            device_id: info_table.device_id,
            uid64: info_table.uid64,
        })
    }

    /// Reset target
    pub fn reset_target(&self, reset_mode: crate::probe::ResetMode) -> CubeProgrammerResult<()> {
        self.check_connection()?;
        api_types::ReturnCode::<0>::from(unsafe { self.api.reset(reset_mode.into()) })
            .check(crate::error::Action::Reset)
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
                    action: crate::error::Action::DownloadFile,
                    message: "Invalid intelhex file".to_string(),
                })?;

            let reader = ihex::Reader::new_with_options(
                file_content,
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
                            action: crate::error::Action::DownloadFile,
                            message: format!("Invalid intelhex file: {}", e),
                        });
                    }
                }
            }
        }

        self.check_connection()?;

        let file_path = utility::path_to_widestring(file_path);

        api_types::ReturnCode::<0>::from(unsafe {
            self.api.downloadFile(
                file_path?.as_ptr(),
                0,
                if skip_erase { 1 } else { 0 },
                if verify { 1 } else { 0 },
                std::ptr::null(),
            )
        })
        .check(crate::error::Action::DownloadFile)
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
            self.api.downloadFile(
                file_path?.as_ptr(),
                start_address,
                if skip_erase { 1 } else { 0 },
                if verify { 1 } else { 0 },
                std::ptr::null(),
            )
        })
        .check(crate::error::Action::DownloadFile)
    }

    /// Perform mass erase
    pub fn mass_erase(&self) -> CubeProgrammerResult<()> {
        self.check_connection()?;

        api_types::ReturnCode::<0>::from(unsafe { self.api.massErase(std::ptr::null_mut()) })
            .check(crate::error::Action::MassErase)
    }

    /// Save memory to file
    /// Attention: The file path must end with .hex or .bin
    pub fn save_memory(
        &self,
        file_path: impl AsRef<std::path::Path>,
        start_address: u32,
        size_bytes: u32,
    ) -> CubeProgrammerResult<()> {
        self.check_connection()?;

        api_types::ReturnCode::<0>::from(unsafe {
            self.api.saveMemoryToFile(
                i32::try_from(start_address).map_err(|x| CubeProgrammerError::Parameter {
                    action: crate::error::Action::SaveMemory,
                    message: format!("Start address exceeds max value: {}", x),
                })?,
                i32::try_from(size_bytes).map_err(|x| CubeProgrammerError::Parameter {
                    action: crate::error::Action::SaveMemory,
                    message: format!("Size exceeds max value: {}", x),
                })?,
                utility::path_to_widestring(file_path)?.as_ptr(),
            )
        })
        .check(crate::error::Action::SaveMemory)
    }

    /// Enable roud out protection level 1 (0xBB)
    pub fn enable_read_out_protection(&self) -> CubeProgrammerResult<()> {
        /// Command according to Example 3 of the CubeProgrammer API documentation
        const COMMAND_ENABLE_ROP_LEVEL_1: &str = "-ob rdp=0xbb";

        self.check_connection()?;

        api_types::ReturnCode::<0>::from(unsafe {
            self.api.sendOptionBytesCmd(
                utility::string_to_cstring(COMMAND_ENABLE_ROP_LEVEL_1)?.as_ptr()
                    as *mut std::ffi::c_char,
            )
        })
        .check(crate::error::Action::EnableReadOutProtection)
    }

    /// Disable read out protection
    /// Attention: This command will eOrase the device memory
    pub fn disable_read_out_protection(&self) -> CubeProgrammerResult<()> {
        self.check_connection()?;
        api_types::ReturnCode::<0>::from(unsafe { self.api.readUnprotect() })
            .check(crate::error::Action::DisableReadOutProtection)?;
        Ok(())
    }

    /// Check connection to target
    /// Consumes self and and only returns self if the connection is still maintained
    /// If the connection is lost, the user is forced to reconnect
    fn check_connection(&self) -> CubeProgrammerResult<()> {
        api_types::ReturnCode::<1>::from(unsafe { self.api.checkDeviceConnection() })
            .check(crate::error::Action::CheckConnection)
    }

    /// Read in bytes from memory
    ///
    /// # Arguments
    /// address: Start address to read from
    /// count: Number of bytes to read
    pub fn read_memory_bytes(&self, address: u32, count: usize) -> CubeProgrammerResult<Vec<u8>> {
        let mut data = std::ptr::null_mut();
        let size = u32::try_from(count).map_err(|x| CubeProgrammerError::Parameter {
            action: crate::error::Action::ReadMemory,
            message: format!("Size exceeds max value: {}", x),
        })?;

        api_types::ReturnCode::<0>::from(unsafe { self.api.readMemory(address, &mut data, size) })
            .check(crate::error::Action::ReadMemory)?;

        if data.is_null() {
            return Err(CubeProgrammerError::ActionOutputUnexpected {
                action: crate::error::Action::ReadMemory,
                unexpected_output: crate::error::UnexpectedOutput::Null,
            });
        }

        let vec = unsafe { std::slice::from_raw_parts_mut(data, count) }.to_vec();

        unsafe {
            self.api.freeLibraryMemory(data as *mut std::ffi::c_void);
        }

        Ok(vec)
    }

    /// Write memory as bytes
    ///
    /// # Arguments
    /// address: Start address to write to
    /// data: Data to write
    pub fn write_memory_bytes(&self, address: u32, data: &[u8]) -> CubeProgrammerResult<()> {
        let size = u32::try_from(data.len()).map_err(|x| CubeProgrammerError::Parameter {
            action: crate::error::Action::WriteMemory,
            message: format!("Size exceeds max value: {}", x),
        })?;

        api_types::ReturnCode::<0>::from(unsafe {
            self.api.writeMemory(address, data.as_ptr() as *mut _, size)
        })
        .check(crate::error::Action::WriteMemory)
    }

    /// Read memory as struct
    /// The struct needs to support the traits `bytemuck::Pod` and `bytemuck::Zeroable`
    /// These traits are implemented for lots of types e.g. (full list available [here](https://docs.rs/bytemuck/1.21.0/bytemuck/trait.Pod.html)):
    /// - u8, u16, u32
    /// - i8, i16, i32
    /// - f32
    ///
    /// # Arguments
    /// address: Start address to read from
    /// count: Number of struct elements to read
    pub fn read_memory<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        address: u32,
        count: usize,
    ) -> CubeProgrammerResult<Vec<T>> {
        let size = u32::try_from(std::mem::size_of::<T>() * count).map_err(|x| {
            CubeProgrammerError::Parameter {
                action: crate::error::Action::ReadMemory,
                message: format!("Size exceeds max value: {}", x),
            }
        })?;

        let mut data = std::ptr::null_mut();

        api_types::ReturnCode::<0>::from(unsafe { self.api.readMemory(address, &mut data, size) })
            .check(crate::error::Action::ReadMemory)?;

        if data.is_null() {
            return Err(CubeProgrammerError::ActionOutputUnexpected {
                action: crate::error::Action::ReadMemory,
                unexpected_output: crate::error::UnexpectedOutput::Null,
            });
        }

        let pod_data: &[T] =
            bytemuck::try_cast_slice(unsafe { std::slice::from_raw_parts(data, size as _) })
                .map_err(|_| CubeProgrammerError::ActionOutputUnexpected {
                    action: crate::error::Action::ReadMemory,
                    unexpected_output: crate::error::UnexpectedOutput::SliceConversion,
                })?;

        let pod_data = pod_data.to_vec();

        unsafe {
            self.api.freeLibraryMemory(data as *mut std::ffi::c_void);
        }

        if pod_data.len() != count {
            return Err(CubeProgrammerError::ActionOutputUnexpected {
                action: crate::error::Action::ReadMemory,
                unexpected_output: crate::error::UnexpectedOutput::SliceLength,
            });
        }

        Ok(pod_data)
    }

    /// Write memory as struct
    /// The struct needs to support the traits `bytemuck::Pod` and `bytemuck::Zeroable`
    /// These traits are implemented for lots of types e.g. (full list available [here](https://docs.rs/bytemuck/1.21.0/bytemuck/trait.Pod.html)):
    /// - u8, u16, u32
    /// - i8, i16, i32
    /// - f32
    ///
    /// # Arguments
    /// address: Start address to write to
    /// data: A slice of struct elements to write
    pub fn write_memory<T: bytemuck::Pod + std::fmt::Debug>(
        &self,
        address: u32,
        data: &[T],
    ) -> CubeProgrammerResult<()> {
        let size = u32::try_from(std::mem::size_of_val(data)).map_err(|x| {
            CubeProgrammerError::Parameter {
                action: crate::error::Action::WriteMemory,
                message: format!("Size exceeds max value: {}", x),
            }
        })?;

        let mut bytes = data
            .iter()
            .flat_map(|x| bytemuck::bytes_of(x).to_vec())
            .collect::<Vec<_>>();

        api_types::ReturnCode::<0>::from(unsafe {
            self.api
                .writeMemory(address, bytes.as_mut_ptr() as *mut i8, size)
        })
        .check(crate::error::Action::WriteMemory)
    }

    /// Start the wireless stack
    pub fn start_wireless_stack(&self) -> CubeProgrammerResult<()> {
        self.supports_fus()?;

        api_types::ReturnCode::<1>::from(unsafe { self.api.startWirelessStack() })
            .check(crate::error::Action::StartWirelessStack)
    }
}

impl ConnectedFusCubeProgrammer<'_> {
    pub fn fus_info(&self) -> &crate::fus::Information {
        &self.fus_info
    }

    pub fn delete_wireless_stack(&self) -> CubeProgrammerResult<()> {
        api_types::ReturnCode::<1>::from(unsafe { self.programmer.api.firmwareDelete() })
            .check(crate::error::Action::DeleteWirelessStack)
    }

    pub fn upgrade_wireless_stack(
        &self,
        file_path: impl AsRef<std::path::Path>,
        start_address: u32,
        first_install: bool,
        verify: bool,
        start_stack_after_update: bool,
    ) -> CubeProgrammerResult<()> {
        self.programmer.check_connection()?;

        api_types::ReturnCode::<1>::from(unsafe {
            self.programmer.api.firmwareUpgrade(
                utility::path_to_widestring(file_path)?.as_ptr(),
                start_address,
                if first_install { 1 } else { 0 },
                if verify { 1 } else { 0 },
                if start_stack_after_update { 1 } else { 0 },
            )
        })
        .check(crate::error::Action::UpgradeWirelessStack)
    }

    pub fn start_wireless_stack(&self) -> CubeProgrammerResult<()> {
        self.programmer.start_wireless_stack()
    }

    pub fn disconnect(self) {
        self.programmer.disconnect()
    }
}
