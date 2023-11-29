use super::ShortcutFile;
use std::{
    ffi::{CString, NulError, OsString},
    iter::once,
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
    sync::Once,
};

use log::debug;
use thiserror::Error;
use windows::{
    core::{ComInterface, PCSTR, PCWSTR},
    Win32::{
        Foundation::TRUE,
        System::Com::{
            CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_INPROC_SERVER,
            COINIT_MULTITHREADED,
        },
        UI::{
            Shell::*,
            WindowsAndMessaging::{SW_HIDE, SW_SHOW},
        },
    },
};

static CO_INITIALIZE_ONCE: Once = Once::new();

fn initialize_com() {
    CO_INITIALIZE_ONCE.call_once(|| unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok();
    })
}

#[derive(Debug, Error)]
pub enum WindowsShortcutError {
    #[error("Path was unable to be converted into a CString. {0:?}")]
    PathToStringError(OsString),
    #[error("String was unable to be converted into a CString. {0}")]
    StringToCStringError(#[from] NulError),
    #[error("Internal Windows Error. {0}")]
    WindowsError(#[from] ::windows::core::Error),
}
/// Saves a Shortcut to a File. Uses the Win32 API.
///
/// I would rather not use the Win32 API.
/// But I don't want to implement the LNK file format myself.
pub fn save_shortcut_file(
    shortcut: ShortcutFile,
    to: impl Into<PathBuf>,
) -> Result<(), WindowsShortcutError> {
    let to = to.into();
    debug!("Creating Shortcut to {:?} at {:?}", shortcut.path, to);
    initialize_com();
    let path = path_to_c_string(shortcut.path)?;
    let description = shortcut.description.map(string_to_c_string).transpose()?;
    let arguments = arguments_to_string(&shortcut.arguments)?;
    let icon = shortcut.icon.map(path_to_c_string).transpose()?;
    let show_cmd = if shortcut.show_terminal {
        SW_SHOW
    } else {
        SW_HIDE
    };
    let working_directory = shortcut
        .working_directory
        .map(path_to_c_string)
        .transpose()?;
    let to = path_to_utf16(to);
    unsafe {
        let shell_link: IShellLinkA = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        shell_link.SetPath(PCSTR(path.as_ptr().cast()))?;
        shell_link.SetArguments(PCSTR(arguments.as_ptr().cast()))?;
        shell_link.SetShowCmd(show_cmd)?;
        if let Some(description) = description {
            shell_link.SetDescription(PCSTR(description.as_ptr().cast()))?;
        }
        if let Some(working_directory) = working_directory {
            shell_link.SetWorkingDirectory(PCSTR(working_directory.as_ptr().cast()))?;
        }
        if let Some(icon) = icon {
            shell_link.SetIconLocation(PCSTR(icon.as_ptr().cast()), 0)?;
        }

        shell_link
            .cast::<IPersistFile>()?
            .Save(PCWSTR(to.as_ptr()), TRUE)?;
    }
    Ok(())
}

pub fn read_shortcut_file(_path: impl Into<PathBuf>) -> Result<ShortcutFile, WindowsShortcutError> {
    todo!("Support reading shortcuts")
}

fn arguments_to_string(arguments: &[String]) -> Result<CString, WindowsShortcutError> {
    let arguments = arguments.join(" ");
    string_to_c_string(arguments)
}

fn string_to_c_string(string: impl Into<Vec<u8>>) -> Result<CString, WindowsShortcutError> {
    CString::new(string).map_err(WindowsShortcutError::from)
}
/// Converts a Path to a CString.
///
/// Path must be UTF-8
fn path_to_c_string(path: impl AsRef<Path>) -> Result<CString, WindowsShortcutError> {
    let path = path
        .as_ref()
        .to_str()
        .ok_or(WindowsShortcutError::PathToStringError(
            path.as_ref().as_os_str().to_os_string(),
        ))?;
    CString::new(path).map_err(WindowsShortcutError::from)
}
fn path_to_utf16(path: PathBuf) -> Vec<u16> {
    let path = path.into_os_string();
    return path.encode_wide().chain(once(0)).collect::<Vec<u16>>();
}
