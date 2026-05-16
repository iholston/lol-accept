use std::env;
use std::io;
use std::path::PathBuf;

use winreg::enums::*;
use winreg::RegKey;

const APP_NAME: &str = "LoL-Accept";
const RUN_REGISTRY_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

fn get_executable_path() -> io::Result<String> {
    let path: PathBuf = env::current_exe()?;
    path.into_os_string()
        .into_string()
        .map_err(|_| io::Error::other("Failed to convert path to string"))
}

fn get_startup_command() -> io::Result<String> {
    Ok(format!("\"{}\"", get_executable_path()?))
}

pub fn add_to_startup() -> io::Result<()> {
    let app_command = get_startup_command()?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run_key, _) = hkcu.create_subkey(RUN_REGISTRY_PATH)?;
    run_key.set_value(APP_NAME, &app_command)?;
    Ok(())
}

pub fn remove_from_startup() -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu.open_subkey_with_flags(RUN_REGISTRY_PATH, KEY_WRITE)?;
    match run_key.delete_value(APP_NAME) {
        Ok(_) => Ok(()),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn is_in_startup() -> io::Result<bool> {
    let current_command = get_startup_command()?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    match hkcu.open_subkey_with_flags(RUN_REGISTRY_PATH, KEY_READ) {
        Ok(run_key) => match run_key.get_value::<String, _>(APP_NAME) {
            Ok(stored_command) => Ok(stored_command == current_command),
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        },
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e),
    }
}

pub fn cleanup_stale_registry() -> io::Result<bool> {
    let current_command = get_startup_command()?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let run_key = match hkcu.open_subkey_with_flags(RUN_REGISTRY_PATH, KEY_READ | KEY_WRITE) {
        Ok(run_key) => run_key,
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };

    match run_key.get_value::<String, _>(APP_NAME) {
        Ok(stored_command) => {
            if stored_command != current_command {
                run_key.delete_value(APP_NAME)?;
                return Ok(true);
            }
        }
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            // No entry exists, nothing to clean.
        }
        Err(e) => return Err(e),
    }

    Ok(false)
}
