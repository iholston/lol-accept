use std::time::Duration;
use lazy_static::lazy_static;

lazy_static! {
    static ref CLIENT: reqwest::blocking::Client = {
        reqwest::blocking::Client::builder()
            .use_rustls_tls()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(2))
            .no_proxy()
            .build()
            .unwrap()
    };
}

pub fn make_client() -> &'static reqwest::blocking::Client {
    &CLIENT
}

pub fn accept_match(endpoint: &str) {
    let client = make_client();
    let url = format!("{endpoint}/lol-matchmaking/v1/ready-check/accept");
    let _ = client
        .post(url)
        .version(reqwest::Version::HTTP_2)
        .header(reqwest::header::ACCEPT, "application/json")
        .send();
}

pub fn get_phase(endpoint: &str) -> String {
    let mut phase = String::from("Unknown");
    let url = format!("{endpoint}/lol-gameflow/v1/gameflow-phase");
    let client = make_client();
    let response = client
        .get(url)
        .version(reqwest::Version::HTTP_2)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .ok();
    if let Some(response) = response {
        phase = response.text().unwrap_or(phase);
        phase = phase.trim_matches('"').to_string();
    }
    phase
}
