use std::sync::LazyLock;

const LCU_PORT_KEY: &str = "--app-port=";
const LCU_TOKEN_KEY: &str = "--remoting-auth-token=";
const LCU_COMMAND: &str = "Get-CimInstance Win32_Process -Filter \"name = 'LeagueClientUx.exe'\" | Select-Object -ExpandProperty CommandLine";

static PORT_REGEXP: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"--app-port=\d+").unwrap());
static TOKEN_REGEXP: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"--remoting-auth-token=\S+").unwrap());

#[derive(Debug, Clone)]
pub struct LcuAuth {
    pub base_url: String,
    pub token: String,
}

pub fn discover() -> Option<LcuAuth> {
    use powershell_script::PsScriptBuilder;

    let ps = PsScriptBuilder::new()
        .no_profile(true)
        .non_interactive(true)
        .hidden(true)
        .print_commands(false)
        .build();

    match ps.run(LCU_COMMAND) {
        Ok(out) => out.stdout().and_then(|output| match_stdout(&output)),
        Err(err) => {
            println!("cmd error: {:?}", err);
            None
        }
    }
}

fn match_stdout(stdout: &str) -> Option<LcuAuth> {
    let port = PORT_REGEXP.find(stdout)?.as_str().replace(LCU_PORT_KEY, "");
    let token = TOKEN_REGEXP
        .find(stdout)?
        .as_str()
        .replace(LCU_TOKEN_KEY, "")
        .replace(['\\', '\"'], "");

    let base_url = format!("https://127.0.0.1:{port}");

    Some(LcuAuth { base_url, token })
}
