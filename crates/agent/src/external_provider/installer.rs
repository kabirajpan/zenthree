use std::path::PathBuf;

/// Path where Zenthree stores its own managed external agent binaries.
pub fn zenthree_antigravity_dir() -> PathBuf {
    if let Some(dir) = dirs::data_local_dir() {
        dir.join("zenthree").join("external_agents").join("antigravity-acp")
    } else if let Some(home) = dirs::home_dir() {
        home.join(".local").join("share").join("zenthree").join("external_agents").join("antigravity-acp")
    } else {
        PathBuf::from("external_agents").join("antigravity-acp")
    }
}

/// Look up the official Google Antigravity ACP server binary download URL for the current OS and CPU architecture.
pub fn get_antigravity_download_url() -> Result<(&'static str, &'static str), String> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        Ok((
            "https://dl.google.com/agy-extensions/releases/linux/agy-acp-server-1.2.1-linux-x86_64.zip",
            "agy_acp_server.par",
        ))
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        Ok((
            "https://dl.google.com/agy-extensions/releases/linux/agy-acp-server-1.2.1-linux-arm64.zip",
            "agy_acp_server.par",
        ))
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        Ok((
            "https://dl.google.com/agy-extensions/releases/macos/agy-acp-server-1.2.1-darwin-arm64.zip",
            "agy_acp_server.par",
        ))
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        Ok((
            "https://dl.google.com/agy-extensions/releases/macos/agy-acp-server-1.2.1-darwin-x86_64.zip",
            "agy_acp_server.par",
        ))
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        Ok((
            "https://dl.google.com/agy-extensions/releases/windows/agy-acp-server-1.2.1-windows-x86_64.zip",
            "agy_acp_server.exe",
        ))
    }
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        Ok((
            "https://dl.google.com/agy-extensions/releases/windows/agy-acp-server-1.2.1-windows-arm64.zip",
            "agy_acp_server.exe",
        ))
    }
    #[cfg(not(any(
        all(target_os = "linux", any(target_arch = "x86_64", target_arch = "aarch64")),
        all(target_os = "macos", any(target_arch = "x86_64", target_arch = "aarch64")),
        all(target_os = "windows", any(target_arch = "x86_64", target_arch = "aarch64")),
    )))]
    {
        Err("Unsupported operating system or architecture for Google Antigravity ACP.".to_string())
    }
}

/// Download and extract Google Antigravity ACP server directly into Zenthree's external agents directory.
pub fn install_antigravity<F>(progress_callback: F) -> Result<PathBuf, String>
where
    F: Fn(&str, f32) + Send + Sync,
{
    let (url, exec_name) = get_antigravity_download_url()?;
    let install_dir = zenthree_antigravity_dir();

    progress_callback("Connecting to Google download server...", 0.0);
    std::fs::create_dir_all(&install_dir).map_err(|e| {
        format!("Failed to create directory {}: {e}", install_dir.display())
    })?;

    progress_callback("Connecting to download server...", 0.02);
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let mut response = client
        .get(url)
        .send()
        .map_err(|e| format!("Failed to download Antigravity from Google ({url}): {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status {} from {url}",
            response.status()
        ));
    }

    let total_size = response.content_length().unwrap_or(960_000_000);
    let mut downloaded: u64 = 0;
    let mut bytes = Vec::with_capacity(if total_size < 1_500_000_000 { total_size as usize } else { 0 });
    let mut buffer = [0u8; 65536];

    use std::io::Read;
    let mut last_report = std::time::Instant::now();

    loop {
        let n = response.read(&mut buffer).map_err(|e| format!("Failed to read stream: {e}"))?;
        if n == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..n]);
        downloaded += n as u64;

        if last_report.elapsed().as_millis() >= 100 || downloaded == total_size {
            last_report = std::time::Instant::now();
            let ratio = (downloaded as f32 / total_size as f32).clamp(0.0, 1.0);
            // 0.05 to 0.85 progress range for downloading
            let prog = 0.05 + ratio * 0.80;
            let mb_done = downloaded as f32 / 1_048_576.0;
            let mb_total = total_size as f32 / 1_048_576.0;
            progress_callback(
                &format!("Downloading: {:.0} MB / {:.0} MB ({:.0}%)", mb_done, mb_total, ratio * 100.0),
                prog,
            );
        }
    }

    progress_callback("Extracting server archive...", 0.88);
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Failed to read downloaded zip archive: {e}"))?;

    let total_files = archive.len();
    for i in 0..total_files {
        if i % 20 == 0 || i + 1 == total_files {
            let ratio = i as f32 / total_files as f32;
            let prog = 0.88 + ratio * 0.10;
            progress_callback(&format!("Extracting files ({}/{})", i + 1, total_files), prog);
        }

        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read entry {i} from archive: {e}"))?;

        let outpath = match file.enclosed_name() {
            Some(path) => install_dir.join(path),
            None => continue,
        };

        if file.is_dir() {
            std::fs::create_dir_all(&outpath).map_err(|e| {
                format!("Failed to create extraction dir {}: {e}", outpath.display())
            })?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    let _ = std::fs::create_dir_all(p);
                }
            }
            let mut outfile = std::fs::File::create(&outpath).map_err(|e| {
                format!("Failed to write file {}: {e}", outpath.display())
            })?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| {
                format!("Failed to copy file content {}: {e}", outpath.display())
            })?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = file.unix_mode().unwrap_or(0o755);
                // Ensure executable permissions are granted for binaries
                let _ = std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode | 0o111));
            }
        }
    }

    // Locate the expected executable
    let candidate = install_dir.join(exec_name);
    if candidate.exists() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&candidate, std::fs::Permissions::from_mode(0o755));
        }
        progress_callback("Installation complete!", 1.0);
        Ok(candidate)
    } else {
        // Fallback: search recursively in the install dir
        if let Some(found) = find_binary_in_tree(&install_dir, exec_name) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&found, std::fs::Permissions::from_mode(0o755));
            }
            progress_callback("Installation complete!", 1.0);
            Ok(found)
        } else {
            Err(format!(
                "Archive extracted, but {} was not found in {}",
                exec_name,
                install_dir.display()
            ))
        }
    }
}

fn find_binary_in_tree(dir: &std::path::Path, target_name: &str) -> Option<PathBuf> {
    if !dir.exists() {
        return None;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if let Some(found) = find_binary_in_tree(&p, target_name) {
                    return Some(found);
                }
            } else if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if name == target_name {
                    return Some(p);
                }
            }
        }
    }
    None
}

/// Remove the installed Google Antigravity ACP binary directory.
pub fn uninstall_antigravity() -> Result<(), String> {
    let dir = zenthree_antigravity_dir();
    if dir.exists() {
        std::fs::remove_dir_all(&dir)
            .map_err(|e| format!("Failed to remove directory {}: {e}", dir.display()))?;
    }
    Ok(())
}
