//! Rust client for the [check-host.one](https://check-host.one) API: ping,
//! TCP ping, HTTP, traceroute and DNS checks from servers in 100+ countries.
//! No API key is needed.
//!
//! ```no_run
//! let ch = checkhost::Client::new();
//! let m = ch.ping("example.com", 3, &["DE", "US"]).unwrap();
//! println!("https://check-host.one/r/{}", m["resultSlug"].as_str().unwrap());
//! ```

use serde_json::{json, Value};
use std::{fmt, thread, time::Duration};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub enum Error {
    /// An error response. `retry_after` (seconds) is set on 429.
    Api { status: u16, kind: String, message: String, retry_after: Option<u64> },
    Transport(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Api { status, kind, message, .. } => write!(f, "{status} {kind}: {message}"),
            Error::Transport(e) => write!(f, "transport: {e}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Client {
    base_url: String,
    agent: ureq::Agent,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    pub fn new() -> Self {
        Self::with_base_url("https://check-host.one")
    }

    pub fn with_base_url(base_url: &str) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(30))
            .user_agent(&format!("check-host-rust/{VERSION}"))
            .build();
        Client { base_url: base_url.trim_end_matches('/').to_string(), agent }
    }

    fn send(&self, req: ureq::Request, body: Option<&Value>) -> Result<Value> {
        let req = req.set("Accept", "application/json");
        let res = match body {
            Some(b) => req.send_json(b),
            None => req.call(),
        };
        match res {
            Ok(r) => r.into_json().map_err(|e| Error::Transport(e.to_string())),
            Err(ureq::Error::Status(status, r)) => {
                let retry_after = r.header("Retry-After").and_then(|v| v.parse().ok());
                let v: Value = r.into_json().unwrap_or(Value::Null);
                let s = |k: &str| v["error"][k].as_str().unwrap_or_default().to_string();
                Err(Error::Api { status, kind: s("type"), message: s("message"), retry_after })
            }
            Err(e) => Err(Error::Transport(e.to_string())),
        }
    }

    fn get(&self, path: &str) -> Result<Value> {
        self.send(self.agent.get(&(self.base_url.clone() + path)), None)
    }

    /// POST /api/v1/measurements. Returns `{"id", "probesCount"}`.
    pub fn create_measurement(&self, request: &Value) -> Result<Value> {
        self.send(self.agent.post(&(self.base_url.clone() + "/api/v1/measurements")), Some(request))
    }

    pub fn get_measurement(&self, id: &str) -> Result<Value> {
        self.get(&format!("/api/v1/measurements/{}", urlencode(id)))
    }

    /// A saved result, kept for 90 days.
    pub fn get_result(&self, slug: &str) -> Result<Value> {
        self.get(&format!("/api/v1/results/{}", urlencode(slug)))
    }

    /// ASN, operator and location. `None` looks up the caller's own address.
    pub fn host_info(&self, target: Option<&str>) -> Result<Value> {
        match target {
            Some(t) => self.get(&format!("/api/v1/hostinfo?target={}", urlencode(t))),
            None => self.get("/api/v1/hostinfo"),
        }
    }

    pub fn probes(&self) -> Result<Value> {
        self.get("/api/v1/probes")
    }

    pub fn stats(&self) -> Result<Value> {
        self.get("/api/v1/stats")
    }

    /// Start a check and poll once a second until it finishes (max ~60 s).
    /// `locations` are free-form: country code, city, continent, ASN or network.
    pub fn run(&self, kind: &str, target: &str, limit: u32, locations: &[&str], options: Value) -> Result<Value> {
        let locs: Vec<Value> = locations.iter().map(|l| json!({ "magic": l })).collect();
        let mut req = json!({ "type": kind, "target": target, "limit": limit, "measurementOptions": options });
        if !locs.is_empty() {
            req["locations"] = Value::Array(locs);
        }
        let id = self.create_measurement(&req)?["id"].as_str().unwrap_or_default().to_string();
        for _ in 0..60 {
            let m = self.get_measurement(&id)?;
            if m["status"] != "in-progress" {
                return Ok(m);
            }
            thread::sleep(Duration::from_secs(1));
        }
        self.get_measurement(&id)
    }

    pub fn ping(&self, target: &str, limit: u32, locations: &[&str]) -> Result<Value> {
        self.run("ping", target, limit, locations, json!({ "packets": 3 }))
    }

    pub fn tcp_ping(&self, target: &str, port: u16, limit: u32, locations: &[&str]) -> Result<Value> {
        self.run("ping", target, limit, locations, json!({ "protocol": "TCP", "port": port }))
    }

    pub fn http(&self, target: &str, limit: u32, locations: &[&str]) -> Result<Value> {
        let opts = json!({ "protocol": "HTTPS", "request": { "method": "HEAD", "path": "/" } });
        self.run("http", target, limit, locations, opts)
    }

    pub fn traceroute(&self, target: &str, limit: u32, locations: &[&str]) -> Result<Value> {
        self.run("traceroute", target, limit, locations, json!({ "protocol": "ICMP" }))
    }

    pub fn dns(&self, target: &str, record: &str, limit: u32, locations: &[&str]) -> Result<Value> {
        self.run("dns", target, limit, locations, json!({ "query": { "type": record } }))
    }
}

fn urlencode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}
