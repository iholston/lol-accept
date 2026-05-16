use std::sync::LazyLock;
use std::time::Duration;

use crate::cmd::LcuAuth;

static CLIENT: LazyLock<reqwest::blocking::Client> = LazyLock::new(|| {
    reqwest::blocking::Client::builder()
        .use_rustls_tls()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(2))
        .no_proxy()
        .build()
        .unwrap()
});

pub fn make_client() -> &'static reqwest::blocking::Client {
    &CLIENT
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameflowPhase {
    Lobby,
    Matchmaking,
    ReadyCheck,
    Unknown,
}

impl GameflowPhase {
    fn from_lcu_response(value: &str) -> Self {
        match value.trim_matches('"') {
            "Lobby" => Self::Lobby,
            "Matchmaking" => Self::Matchmaking,
            "ReadyCheck" => Self::ReadyCheck,
            _ => Self::Unknown,
        }
    }
}

pub fn accept_match(auth: &LcuAuth) {
    let client = make_client();
    let url = format!("{}/lol-matchmaking/v1/ready-check/accept", auth.base_url);
    let _ = client
        .post(url)
        .version(reqwest::Version::HTTP_2)
        .basic_auth("riot", Some(&auth.token))
        .header(reqwest::header::ACCEPT, "application/json")
        .send();
}

pub fn get_phase(auth: &LcuAuth) -> GameflowPhase {
    let url = format!("{}/lol-gameflow/v1/gameflow-phase", auth.base_url);
    let client = make_client();
    let response = client
        .get(url)
        .version(reqwest::Version::HTTP_2)
        .basic_auth("riot", Some(&auth.token))
        .header(reqwest::header::ACCEPT, "application/json")
        .send();

    match response {
        Ok(response) => {
            let phase = response.text().unwrap_or_default();
            GameflowPhase::from_lcu_response(&phase)
        }
        Err(_) => GameflowPhase::Unknown,
    }
}
