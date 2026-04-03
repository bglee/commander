use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const TRUST_FILE: &str = "commander/trusted.json";
const PROJECT_FILE: &str = ".commander.json";

#[derive(Serialize, Deserialize, Default)]
pub struct TrustStore {
    trusted: HashMap<String, String>,
}

impl TrustStore {
    pub fn load() -> Self {
        let Some(path) = Self::store_path() else {
            return Self::default();
        };
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(path) = Self::store_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    pub fn trust(&mut self, path: &str, hash: &str) {
        self.trusted.insert(path.to_string(), hash.to_string());
        self.save();
    }

    pub fn is_trusted(&self, path: &str, hash: &str) -> bool {
        self.trusted.get(path).is_some_and(|h| h == hash)
    }

    fn store_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join(TRUST_FILE))
    }
}

pub enum TrustStatus {
    NoFile,
    Trusted { contents: String },
    Untrusted { path: String, contents: String, hash: String },
}

fn sha256_hex(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn check_trust() -> TrustStatus {
    let path = Path::new(PROJECT_FILE);
    if !path.exists() {
        return TrustStatus::NoFile;
    }

    let Ok(contents) = fs::read_to_string(path) else {
        return TrustStatus::NoFile;
    };

    let abs_path = match fs::canonicalize(path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => return TrustStatus::NoFile,
    };

    let hash = sha256_hex(&contents);
    let store = TrustStore::load();

    if store.is_trusted(&abs_path, &hash) {
        TrustStatus::Trusted { contents }
    } else {
        TrustStatus::Untrusted {
            path: abs_path,
            contents,
            hash,
        }
    }
}
