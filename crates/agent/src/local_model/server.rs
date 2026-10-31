use std::path::PathBuf;
use std::process::Child;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use crate::local_model::manager::LocalModelManager;

static MANAGED_SERVER_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
static MANAGED_MODEL_NAME: Mutex<Option<String>> = Mutex::new(None);

pub struct LocalServerManager;

impl LocalServerManager {
    /// Return the directory where local runner binaries are stored
    pub fn runners_dir() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".zenthree").join("bin")
    }

    /// Check if a llama-server binary is installed (cross-platform: Linux, Windows, macOS)
    pub fn runner_bin_path() -> Option<PathBuf> {
        let bin_names = if cfg!(windows) {
            ["llama-server.exe", "llama-server"]
        } else {
            ["llama-server", "llama-server.exe"]
        };

        // 1. ~/.zenthree/bin/llama/llama-server(.exe)
        for name in &bin_names {
            let p = Self::runners_dir().join("llama").join(name);
            if p.is_file() {
                return Some(p);
            }
        }

        // 2. ~/.zenthree/bin/llama-server(.exe)
        for name in &bin_names {
            let p = Self::runners_dir().join(name);
            if p.is_file() {
                return Some(p);
            }
        }

        // 3. System PATH
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_var) {
                for name in &bin_names {
                    let candidate = dir.join(name);
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }
        }

        None
    }

    /// Returns true if llama-server runner is ready to execute
    pub fn is_runner_installed() -> bool {
        Self::runner_bin_path().is_some()
    }

    /// Probes known local LLM endpoints (Ollama, llama-server, LM Studio, etc.)
    pub fn probe_active_endpoint() -> Option<String> {
        let endpoints = [
            "http://127.0.0.1:8080/v1",  // llama-server
            "http://127.0.0.1:11434/v1", // Ollama
            "http://127.0.0.1:1234/v1",  // LM Studio
            "http://127.0.0.1:5000/v1",  // LocalAI / vLLM
        ];

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_millis(350))
            .build()
            .ok()?;

        for ep in endpoints {
            let models_url = format!("{}/models", ep.trim_end_matches('/'));
            if let Ok(resp) = client.get(&models_url).send() {
                if resp.status().is_success() {
                    return Some(ep.to_string());
                }
            }
        }

        None
    }

    /// Returns true if an inference server is currently listening and responding
    pub fn is_running() -> bool {
        Self::probe_active_endpoint().is_some()
    }

    /// Returns the model identifier currently running in the managed server, if any
    pub fn current_running_model() -> Option<String> {
        MANAGED_MODEL_NAME.lock().unwrap().clone()
    }

    /// Stop any managed llama-server process
    pub fn stop_server() {
        let mut guard = MANAGED_SERVER_PROCESS.lock().unwrap();
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        *MANAGED_MODEL_NAME.lock().unwrap() = None;
    }

    /// Start llama-server for a specific downloaded model
    pub fn start_server_for_model(model_id_or_file: &str) -> Result<String, String> {
        // 1. If server is already running for this model, return active endpoint
        if let Some(ep) = Self::probe_active_endpoint() {
            let cur = Self::current_running_model();
            if cur.as_deref() == Some(model_id_or_file) {
                return Ok(ep);
            }
        }

        // 2. Resolve model file path on disk
        let model_path = LocalModelManager::get_model_path(model_id_or_file)
            .or_else(|| {
                // If not found directly, pick first downloaded model
                let downloaded = LocalModelManager::list_installed();
                downloaded.into_iter().next().map(|m| m.file_path)
            })
            .ok_or_else(|| {
                format!(
                    "No downloaded model file found for '{}'. Please download a model from the catalog first.",
                    model_id_or_file
                )
            })?;

        // 3. Locate llama-server binary
        let bin_path = Self::runner_bin_path().ok_or_else(|| {
            "llama-server binary was not found in ~/.zenthree/bin/llama/ or system PATH. Please ensure the local runner is installed.".to_string()
        })?;

        // 4. Terminate any previous managed server
        Self::stop_server();

        // 5. Spawn llama-server on port 8080
        let bin_dir = bin_path.parent().map(|p| p.to_path_buf());
        let mut cmd = std::process::Command::new(&bin_path);
        let n_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        cmd.arg("-m")
            .arg(&model_path)
            .arg("--port")
            .arg("8080")
            .arg("--host")
            .arg("127.0.0.1")
            .arg("-c")
            .arg("16384")
            .arg("-np")
            .arg("1")
            .arg("-t")
            .arg(n_threads.to_string())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        if let Some(ref dir) = bin_dir {
            cmd.current_dir(dir);
            #[cfg(unix)]
            cmd.env("LD_LIBRARY_PATH", dir);
            #[cfg(windows)]
            {
                if let Ok(old_path) = std::env::var("PATH") {
                    cmd.env("PATH", format!("{};{}", dir.display(), old_path));
                }
            }
        }

        let child = cmd.spawn().map_err(|e| format!("Failed to spawn llama-server: {e}"))?;
        *MANAGED_SERVER_PROCESS.lock().unwrap() = Some(child);
        *MANAGED_MODEL_NAME.lock().unwrap() = Some(model_id_or_file.to_string());

        // 6. Wait up to 10 seconds for the server to become responsive
        let start_time = Instant::now();
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_millis(500))
            .build()
            .map_err(|e| e.to_string())?;

        let probe_url = "http://127.0.0.1:8080/v1/models";
        while start_time.elapsed() < Duration::from_secs(12) {
            std::thread::sleep(Duration::from_millis(300));
            if let Ok(resp) = client.get(probe_url).send() {
                if resp.status().is_success() {
                    return Ok("http://127.0.0.1:8080/v1".to_string());
                }
            }
        }

        // If it timed out, check if process died
        let mut guard = MANAGED_SERVER_PROCESS.lock().unwrap();
        if let Some(ref mut c) = *guard {
            if let Ok(Some(status)) = c.try_wait() {
                return Err(format!("llama-server exited prematurely with status: {status}"));
            }
        }

        Err("llama-server was started but did not respond on port 8080 within 12 seconds.".to_string())
    }

    /// Automatically ensure an inference server is ready for the requested model
    pub fn ensure_server_running(model_id_or_file: &str) -> Result<String, String> {
        // 1. If an external server (like Ollama or existing llama-server) is already healthy, use it
        if let Some(ep) = Self::probe_active_endpoint() {
            return Ok(ep);
        }

        // 2. Otherwise auto-start our local llama-server
        Self::start_server_for_model(model_id_or_file)
    }
}
