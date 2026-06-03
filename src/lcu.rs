use std::sync::LazyLock;
use std::time::Duration;

use crate::platform::lcu_auth::LcuAuth;

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
    None,
    Lobby,
    Matchmaking,
    ReadyCheck,
    ChampSelect,
    InProgress,
    Reconnect,
    WaitingForStats,
    PreEndOfGame,
    EndOfGame,
    Unknown,
}

impl GameflowPhase {
    fn from_lcu_response(value: &str) -> Self {
        match value.trim_matches('"') {
            "None" => Self::None,
            "Lobby" => Self::Lobby,
            "Matchmaking" => Self::Matchmaking,
            "ReadyCheck" => Self::ReadyCheck,
            "ChampSelect" => Self::ChampSelect,
            "InProgress" => Self::InProgress,
            "Reconnect" => Self::Reconnect,
            "WaitingForStats" => Self::WaitingForStats,
            "PreEndOfGame" => Self::PreEndOfGame,
            "EndOfGame" => Self::EndOfGame,
            _ => Self::Unknown,
        }
    }
}

pub fn accept_match(auth: &LcuAuth) -> Result<(), reqwest::Error> {
    let client = make_client();
    let url = format!("{}/lol-matchmaking/v1/ready-check/accept", auth.base_url);
    let _ = client
        .post(url)
        .basic_auth("riot", Some(&auth.token))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()?
        .error_for_status()?;

    Ok(())
}

pub fn get_phase(auth: &LcuAuth) -> Result<GameflowPhase, reqwest::Error> {
    let url = format!("{}/lol-gameflow/v1/gameflow-phase", auth.base_url);
    let client = make_client();
    let response = client
        .get(url)
        .basic_auth("riot", Some(&auth.token))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()?
        .error_for_status()?;

    let phase = response.text()?;
    Ok(GameflowPhase::from_lcu_response(&phase))
}
