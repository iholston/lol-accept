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
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to convert path to string"))
}

pub fn add_to_startup() -> io::Result<()> {
    let app_path = get_executable_path()?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run_key, _) = hkcu.create_subkey(RUN_REGISTRY_PATH)?;
    run_key.set_value(APP_NAME, &app_path)?;
    Ok(())
}

pub fn remove_from_startup() -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu.open_subkey_with_flags(RUN_REGISTRY_PATH, KEY_WRITE)?;
    match run_key.delete_value(APP_NAME) {
        Ok(_) => Ok(()),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            Ok(())
        },
        Err(e) => Err(e),
    }
}

pub fn is_in_startup() -> io::Result<bool> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.open_subkey_with_flags(RUN_REGISTRY_PATH, KEY_READ) {
        Ok(run_key) => {
            match run_key.get_value::<String, _>(APP_NAME) {
                Ok(_) => Ok(true),
                Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
                Err(e) => Err(e),
            }
        },
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e),
    }
}

pub fn cleanup_stale_registry() -> io::Result<()> {
    let current_path = get_executable_path()?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu.open_subkey_with_flags(RUN_REGISTRY_PATH, KEY_READ | KEY_WRITE)?;
    match run_key.get_value::<String, _>(APP_NAME) {
        Ok(stored_path) => {
            if stored_path != current_path {
                run_key.delete_value(APP_NAME)?;
            }
        },
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            // No entry exists, nothing to clean
        },
        Err(e) => return Err(e),
    }
    Ok(())
}
