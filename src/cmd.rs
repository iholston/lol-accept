use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

const LCU_PORT_KEY: &str = "--app-port=";
const LCU_TOKEN_KEY: &str = "--remoting-auth-token=";
const LCU_DIR_KEY: &str = "--install-directory=";
const LCU_COMMAND: &str = "Get-CimInstance Win32_Process -Filter \"name = 'LeagueClientUx.exe'\" | Select-Object -ExpandProperty CommandLine";

lazy_static! {
    static ref PORT_REGEXP: regex::Regex = regex::Regex::new(r"--app-port=\d+").unwrap();
    static ref TOKEN_REGEXP: regex::Regex =
        regex::Regex::new(r"--remoting-auth-token=\S+").unwrap();
    static ref DIR_REGEXP: regex::Regex =
        regex::Regex::new(r#"--install-directory=(.*?)""#).unwrap();
    static ref MAC_DIR_REGEXP: regex::Regex =
        regex::Regex::new(r"--install-directory=([^\s]+).*?--").unwrap();
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct LcuAuth {
    pub base_url: String,
    pub token: String,
    pub port: String,
    pub install_dir: String,
}

#[cfg(target_os = "windows")]
pub fn get_lcu_auth() -> LcuAuth {
    use powershell_script::PsScriptBuilder;

    let ps = PsScriptBuilder::new()
        .no_profile(true)
        .non_interactive(true)
        .hidden(true)
        .print_commands(false)
        .build();

    match ps.run(LCU_COMMAND) {
        Ok(out) => {
            if let Some(output) = out.stdout() {
                return match_stdout(&output);
            }
        }
        Err(err) => println!("cmd error: {:?}", err),
    }

    LcuAuth {
        ..Default::default()
    }
}

fn match_stdout(stdout: &str) -> LcuAuth {
    let port = if let Some(port_match) = PORT_REGEXP.find(stdout) {
        port_match.as_str().replace(LCU_PORT_KEY, "")
    } else {
        "0".to_string()
    };

    let token = if let Some(token_match) = TOKEN_REGEXP.find(stdout) {
        token_match
            .as_str()
            .replace(LCU_TOKEN_KEY, "")
            .replace(['\\', '\"'], "")
    } else {
        "".to_string()
    };

    let base_url = make_base_url(&port);

    let raw_dir = if let Some(dir_match) = DIR_REGEXP.find(stdout) {
        dir_match.as_str().replace(LCU_DIR_KEY, "")
    } else {
        "".to_string()
    };
    let output_dir = raw_dir.replace('\"', "");
    let install_dir = format!("{output_dir}/");

    LcuAuth {
        base_url,
        token,
        port,
        install_dir,
    }
}

fn make_base_url(port: &str) -> String {
    format!("https://127.0.0.1:{port}")
}
