use std::path::PathBuf;

pub fn load_gemini_api_key() -> Option<String> {
    if let Ok(val) = std::env::var("GEMINI_API_KEY") {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    let candidates = [
        PathBuf::from(".env"),
        PathBuf::from("apps/zenthree/.env"),
        PathBuf::from("../.env"),
        PathBuf::from("../../.env"),
    ];

    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some((k, v)) = trimmed.split_once('=') {
                    if k.trim() == "GEMINI_API_KEY" {
                        let key = v.trim().trim_matches('"').trim_matches('\'').trim();
                        if !key.is_empty() {
                            return Some(key.to_string());
                        }
                    }
                }
            }
        }
    }

    let fixed_paths = [
        PathBuf::from("/home/kabir/Desktop/projects/personal/framworks/zenthra-v2/.env"),
        PathBuf::from("/home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/.env"),
    ];
    for fixed_path in &fixed_paths {
        if let Ok(content) = std::fs::read_to_string(fixed_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some((k, v)) = trimmed.split_once('=') {
                    if k.trim() == "GEMINI_API_KEY" {
                        let key = v.trim().trim_matches('"').trim_matches('\'').trim();
                        if !key.is_empty() {
                            return Some(key.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

pub fn load_groq_api_key() -> Option<String> {
    if let Ok(val) = std::env::var("GROQ_API_KEY") {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    let candidates = [
        PathBuf::from(".env"),
        PathBuf::from("apps/zenthree/.env"),
        PathBuf::from("../.env"),
        PathBuf::from("../../.env"),
    ];

    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some((k, v)) = trimmed.split_once('=') {
                    if k.trim() == "GROQ_API_KEY" {
                        let key = v.trim().trim_matches('"').trim_matches('\'').trim();
                        if !key.is_empty() {
                            return Some(key.to_string());
                        }
                    }
                }
            }
        }
    }

    let fixed_paths = [
        PathBuf::from("/home/kabir/Desktop/projects/personal/framworks/zenthra-v2/.env"),
        PathBuf::from("/home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/.env"),
    ];
    for fixed_path in &fixed_paths {
        if let Ok(content) = std::fs::read_to_string(fixed_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some((k, v)) = trimmed.split_once('=') {
                    if k.trim() == "GROQ_API_KEY" {
                        let key = v.trim().trim_matches('"').trim_matches('\'').trim();
                        if !key.is_empty() {
                            return Some(key.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}
