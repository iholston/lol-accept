use std::env;
use std::io;
use std::path::PathBuf;

use winreg::enums::*;
use winreg::RegKey;

const APP_NAME: &str = "LoL-Accept";
const RUN_REGISTRY_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

pub fn is_enabled() -> io::Result<bool> {
    // Check if in expected spot
    let current_command = get_startup_command()?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let startup_enabled = match hkcu.open_subkey_with_flags(RUN_REGISTRY_PATH, KEY_READ) {
        Ok(run_key) => match run_key.get_value::<String, _>(APP_NAME) {
            Ok(stored_command) => stored_command == current_command,
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => false,
            Err(e) => return Err(e),
        },
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => false,
        Err(e) => return Err(e),
    };

    // Check if it exists at all and assume if it did that its still enabled
    let removed_stale_entry = cleanup_stale_entry()?;
    if removed_stale_entry {
        enable()?;
    }

    Ok(startup_enabled || removed_stale_entry)
}

pub fn enable() -> io::Result<()> {
    let _ = cleanup_stale_entry();
    let app_command = get_startup_command()?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run_key, _) = hkcu.create_subkey(RUN_REGISTRY_PATH)?;
    run_key.set_value(APP_NAME, &app_command)?;
    Ok(())
}

pub fn disable() -> io::Result<()> {
    let _ = cleanup_stale_entry();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu.open_subkey_with_flags(RUN_REGISTRY_PATH, KEY_WRITE)?;
    match run_key.delete_value(APP_NAME) {
        Ok(_) => Ok(()),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

fn cleanup_stale_entry() -> io::Result<bool> {
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

fn get_executable_path() -> io::Result<String> {
    let path: PathBuf = env::current_exe()?;
    path.into_os_string()
        .into_string()
        .map_err(|_| io::Error::other("Failed to convert path to string"))
}

fn get_startup_command() -> io::Result<String> {
    Ok(format!("\"{}\"", get_executable_path()?))
}
