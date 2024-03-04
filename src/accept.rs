use anyhow::anyhow;
use base64::encode;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use std::sync::{Arc, Mutex};
use std::{fs, path::Path, thread, time::Duration};

#[derive(Clone)]
struct LeagueSession {
    id: u32,
    client: Client,
    addr: String,
}

impl LeagueSession {
    pub fn new(league_pid: u32) -> Result<LeagueSession, Box<dyn std::error::Error>> {
        let mut lockfile_path = String::new();
        unsafe {
            let temp = tasklist::get_proc_path(league_pid);
            let path = Path::new(&temp);
            if let Some(parent) = path.parent() {
                if let Some(parent_str) = parent.to_str() {
                    lockfile_path = format!("{}\\{}", parent_str.to_string(), "lockfile");
                }
            }
        }
        let auth: String;
        let url: String;
        let contents = fs::read_to_string(lockfile_path)?;
        let data: Vec<&str> = contents.split(":").collect();
        if data.len() == 5 {
            auth = format!(
                "Basic {}",
                encode(format!("{}:{}", "riot".to_string(), data[3].to_string()).as_bytes())
            );
            url = format!(
                "{}://{}:{}",
                data[4].to_string(),
                String::from("127.0.0.1"),
                data[2].to_string()
            );
        } else {
            return Err(anyhow!("Could not parse data from lockfile").into());
        }
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_str(&auth)?);
        let session = Client::builder()
            .default_headers(headers)
            .danger_accept_invalid_certs(true)
            .build()?;
        Ok(LeagueSession {
            id: league_pid,
            client: session,
            addr: url,
        })
    }
}

#[derive(Clone)]
pub struct Acceptor {
    game_pname: String,
    game_pid: u32,
    session: Option<LeagueSession>,
    pub paused: Arc<Mutex<bool>>,
    pub terminate: Arc<Mutex<bool>>,
}

impl Acceptor {
    pub fn new() -> Acceptor {
        Acceptor {
            game_pname: String::from("LeagueClient.exe"),
            game_pid: 0,
            session: None,
            paused: Arc::new(Mutex::new(false)),
            terminate: Arc::new(Mutex::new(false)),
        }
    }

    pub fn run(&mut self) {
        while !*self.terminate.lock().unwrap() {
            if !*self.paused.lock().unwrap() && self.game_running() {
                if let Some(_) = self.session.as_ref().filter(|ls| ls.id == self.game_pid) {
                    if self.get_game_phase() == "ReadyCheck" {
                        self.accept_match();
                    }
                } else {
                    match LeagueSession::new(self.game_pid) {
                        Ok(session) => {
                            self.session = Some(session);
                        }
                        Err(_) => {}
                    }
                }
            }
            thread::sleep(Duration::from_secs(3));
        }
    }

    fn game_running(&mut self) -> bool {
        unsafe {
            if let Some(pid) = tasklist::find_process_id_by_name(&self.game_pname).get(0) {
                self.game_pid = *pid;
                return true;
            }
        }
        false
    }

    fn get_game_phase(&self) -> String {
        if let Some(session) = &self.session {
            let url = format!("{}{}", session.addr, "/lol-gameflow/v1/gameflow-phase");
            if let Ok(response) = session.client.get(url).send() {
                if let Some(body) = response.text().ok() {
                    return body.trim_matches('"').to_string();
                }
            }
        }
        String::from("Phase not found")
    }

    fn accept_match(&self) {
        if let Some(session) = &self.session {
            let url = format!(
                "{}{}",
                session.addr, "/lol-matchmaking/v1/ready-check/accept"
            );
            let _ = session.client.post(url).send();
        }
    }
}
