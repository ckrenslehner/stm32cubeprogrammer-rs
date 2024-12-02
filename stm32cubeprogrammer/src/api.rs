use derive_more::{Display, Error};
use log::{debug, info};
use stm32cubeprogrammer_sys::{libloading, CubeProgrammer_API};

#[derive(Debug, Error, Display)]
pub enum CubeProgrammerApiError {
    #[display("LibLoadingError: {}", _0)]
    LibLoadingError(libloading::Error),
}

pub struct CubeProgrammerApi {
    api: stm32cubeprogrammer_sys::CubeProgrammer_API,
}

#[derive(Debug)]
#[repr(transparent)]
pub struct StLink(stm32cubeprogrammer_sys::debugConnectParameters);

impl AsRef<StLink> for stm32cubeprogrammer_sys::debugConnectParameters {
    fn as_ref(&self) -> &StLink {
        unsafe {
            &*(self as *const stm32cubeprogrammer_sys::debugConnectParameters as *const StLink)
        }
    }
}

impl CubeProgrammerApi {
    pub fn new(
        cube_programmer_dir: impl AsRef<std::path::Path>,
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

        let library = Self::load_library(&api_path)
            .map_err(|e| CubeProgrammerApiError::LibLoadingError(e))?;

        let api = unsafe {
            CubeProgrammer_API::from_library(library)
                .map_err(|e| CubeProgrammerApiError::LibLoadingError(e))?
        };

        Ok(Self { api })
    }

    pub fn list_connected_st_link_probes(&self) -> &[StLink] {
        let mut debug_parameters =
            std::ptr::null_mut::<stm32cubeprogrammer_sys::debugConnectParameters>();
        let return_value = unsafe { self.api.getStLinkList(&mut debug_parameters, 0) };

        if return_value < 0 || debug_parameters.is_null() {
            return &[];
        }

        let debug_parameters = unsafe {
            std::slice::from_raw_parts(debug_parameters as *const StLink, return_value as _)
        };
        debug_parameters
    }

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

        Ok(unsafe { load_inner(api_library_path.as_ref()) }?)
    }
}
