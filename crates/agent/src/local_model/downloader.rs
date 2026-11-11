use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use super::catalog::CatalogModel;
use super::manager::LocalModelManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub model_name: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub speed_bytes_per_sec: u64,
    pub eta_seconds: Option<u64>,
    pub is_finished: bool,
    pub error: Option<String>,
}

pub struct ModelDownloader;

impl ModelDownloader {
    /// Synchronously download a catalog model with a progress callback and optional cancellation token.
    pub fn download<F>(
        cat: &CatalogModel,
        cancel_token: Option<Arc<AtomicBool>>,
        mut on_progress: F,
    ) -> Result<PathBuf, String>
    where
        F: FnMut(DownloadProgress),
    {
        let dir = LocalModelManager::ensure_dir().map_err(|e| format!("Failed to create models dir: {e}"))?;
        let final_path = dir.join(&cat.filename);
        let part_path = dir.join(format!("{}.part", &cat.filename));

        // If file already exists and is non-empty, verify or return
        if final_path.is_file() {
            if let Ok(meta) = final_path.metadata() {
                if meta.len() > 10_000_000 {
                    on_progress(DownloadProgress {
                        model_id: cat.id.to_string(),
                        model_name: cat.name.to_string(),
                        downloaded_bytes: meta.len(),
                        total_bytes: meta.len(),
                        percentage: 100.0,
                        speed_bytes_per_sec: 0,
                        eta_seconds: Some(0),
                        is_finished: true,
                        error: None,
                    });
                    let _ = LocalModelManager::record_installed(cat, meta.len());
                    return Ok(final_path);
                }
            }
        }

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(3600))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

        // Determine if we can resume from existing .part
        let mut downloaded_bytes: u64 = 0;
        let mut file = if part_path.exists() {
            let existing_len = part_path.metadata().map(|m| m.len()).unwrap_or(0);
            downloaded_bytes = existing_len;
            OpenOptions::new().create(true).append(true).open(&part_path)
                .map_err(|e| format!("Failed to open part file: {e}"))?
        } else {
            File::create(&part_path).map_err(|e| format!("Failed to create part file: {e}"))?
        };

        let mut req = client.get(cat.download_url)
            .header("User-Agent", "Zenthree-LocalModelDownloader/1.0");

        if downloaded_bytes > 0 {
            req = req.header("Range", format!("bytes={}-", downloaded_bytes));
        }

        let mut response = req.send().map_err(|e| format!("HTTP request failed: {e}"))?;
        let status = response.status();

        if downloaded_bytes > 0 && status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
            // Server doesn't support resuming from this offset; restart from 0
            downloaded_bytes = 0;
            drop(file);
            let _ = fs::remove_file(&part_path);
            file = File::create(&part_path).map_err(|e| format!("Failed to recreate part file: {e}"))?;
            response = client.get(cat.download_url)
                .header("User-Agent", "Zenthree-LocalModelDownloader/1.0")
                .send()
                .map_err(|e| format!("HTTP request restart failed: {e}"))?;
        } else if !status.is_success() && status != reqwest::StatusCode::PARTIAL_CONTENT {
            let _ = fs::remove_file(&part_path);
            return Err(format!("Download failed with HTTP status: {}", status));
        }

        let total_bytes = if let Some(len) = response.content_length() {
            if status == reqwest::StatusCode::PARTIAL_CONTENT {
                downloaded_bytes + len
            } else {
                len
            }
        } else {
            cat.size_bytes
        };

        let _start_time = Instant::now();
        let mut last_emit = Instant::now();
        let mut last_bytes = downloaded_bytes;
        let mut speed = 0u64;

        let mut buffer = [0u8; 64 * 1024]; // 64 KB buffer

        loop {
            if let Some(ref cancel) = cancel_token {
                if cancel.load(Ordering::Relaxed) {
                    drop(file);
                    return Err("Download cancelled by user".to_string());
                }
            }

            let read_bytes = match response.read(&mut buffer) {
                Ok(0) => break, // EOF reached
                Ok(n) => n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(format!("Error reading HTTP stream: {e}")),
            };

            file.write_all(&buffer[..read_bytes])
                .map_err(|e| format!("Error writing to disk: {e}"))?;

            downloaded_bytes += read_bytes as u64;

            let now = Instant::now();
            let elapsed_since_emit = now.duration_since(last_emit).as_secs_f32();
            if elapsed_since_emit >= 0.25 || downloaded_bytes >= total_bytes {
                let bytes_delta = downloaded_bytes.saturating_sub(last_bytes);
                if elapsed_since_emit > 0.05 {
                    speed = (bytes_delta as f32 / elapsed_since_emit) as u64;
                }

                let percentage = if total_bytes > 0 {
                    ((downloaded_bytes as f64 / total_bytes as f64) * 100.0).min(100.0) as f32
                } else {
                    0.0
                };

                let eta_seconds = if speed > 0 && total_bytes > downloaded_bytes {
                    Some((total_bytes - downloaded_bytes) / speed)
                } else {
                    None
                };

                on_progress(DownloadProgress {
                    model_id: cat.id.to_string(),
                    model_name: cat.name.to_string(),
                    downloaded_bytes,
                    total_bytes,
                    percentage,
                    speed_bytes_per_sec: speed,
                    eta_seconds,
                    is_finished: false,
                    error: None,
                });

                last_emit = now;
                last_bytes = downloaded_bytes;
            }
        }

        file.flush().map_err(|e| format!("Failed to flush file: {e}"))?;
        drop(file);

        // Rename .part -> final .gguf file
        if final_path.exists() {
            let _ = fs::remove_file(&final_path);
        }
        fs::rename(&part_path, &final_path)
            .map_err(|e| format!("Failed to rename {} to {}: {}", part_path.display(), final_path.display(), e))?;

        // Update manifest
        let _ = LocalModelManager::record_installed(cat, downloaded_bytes);

        on_progress(DownloadProgress {
            model_id: cat.id.to_string(),
            model_name: cat.name.to_string(),
            downloaded_bytes,
            total_bytes,
            percentage: 100.0,
            speed_bytes_per_sec: 0,
            eta_seconds: Some(0),
            is_finished: true,
            error: None,
        });

        Ok(final_path)
    }

    /// Spawn model download in a background thread
    pub fn download_in_background<F>(
        cat: CatalogModel,
        on_progress: F,
    ) -> (Arc<AtomicBool>, std::thread::JoinHandle<Result<PathBuf, String>>)
    where
        F: Fn(DownloadProgress) + Send + 'static,
    {
        let cancel = Arc::new(AtomicBool::new(false));
        let cancel_thread = cancel.clone();

        let handle = std::thread::spawn(move || {
            Self::download(&cat, Some(cancel_thread), on_progress)
        });

        (cancel, handle)
    }
}
