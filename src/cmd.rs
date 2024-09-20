use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;

const LCU_PORT_KEY: &str = "--app-port=";
const LCU_TOKEN_KEY: &str = "--remoting-auth-token=";
const LCU_DIR_KEY: &str = "--install-directory=";
const LCU_COMMAND: &str = "Get-CimInstance Win32_Process -Filter \"name = 'LeagueClientUx.exe'\" | Select-Object -ExpandProperty CommandLine";

lazy_static! {
    static ref PORT_REGEXP: regex::Regex = regex::Regex::new(r"--app-port=\d+").unwrap();
    static ref TOKEN_REGEXP: regex::Regex = regex::Regex::new(r"--remoting-auth-token=\S+").unwrap();
    static ref DIR_REGEXP: regex::Regex = regex::Regex::new(r#"--install-directory=(.*?)""#).unwrap();
    static ref MAC_DIR_REGEXP: regex::Regex = regex::Regex::new(r"--install-directory=([^\s]+).*?--").unwrap();
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CommandLineOutput {
    pub auth_url: String,
    pub token: String,
    pub port: String,
    pub dir: String,
}

#[cfg(target_os = "windows")]
pub fn get_commandline() -> CommandLineOutput {
    use powershell_script::PsScriptBuilder;

    let ps = PsScriptBuilder::new()
        .no_profile(true)
        .non_interactive(true)
        .hidden(true)
        .print_commands(false)
        .build();

    match ps.run(LCU_COMMAND) {
        Ok(out) => {
            let output = out.stdout();

            if output.is_some() {
                return match_stdout(&String::from(output.unwrap()));
            }
        }
        Err(err) => println!("cmd error: {:?}", err),
    }

    CommandLineOutput {
        ..Default::default()
    }
}

fn match_stdout(stdout: &str) -> CommandLineOutput {
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

    let auth_url = make_auth_url(&token, &port);

    let raw_dir = if let Some(dir_match) = DIR_REGEXP.find(stdout) {
        dir_match.as_str().replace(LCU_DIR_KEY, "")
    } else {
        "".to_string()
    };
    let output_dir = raw_dir.replace('\"', "");
    let dir = format!("{output_dir}/");

    CommandLineOutput {
        auth_url,
        token,
        port,
        dir,
    }
}

fn make_auth_url(token: &String, port: &String) -> String {
    format!("https://riot:{token}@127.0.0.1:{port}")
}
