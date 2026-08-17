/// Locate `"id": N` for an asset whose `"name"` equals `filename` in release JSON.
fn find_asset_id(release_json: &str, filename: &str) -> Option<u64> {
    let needle = format!("\"name\": \"{filename}\"");
    let needle_alt = format!("\"name\":\"{filename}\"");
    let pos = release_json
        .find(&needle)
        .or_else(|| release_json.find(&needle_alt))?;
    // Search a window around the name for the asset id (id usually appears before name).
    let start = pos.saturating_sub(400);
    let end = (pos + filename.len() + 200).min(release_json.len());
    let window = &release_json[start..end];
    // Prefer the last "id": digits before the name match (closest preceding id).
    let rel = pos - start;
    let before = &window[..rel];
    let mut last_id = None;
    let mut i = 0;
    let b = before.as_bytes();
    while i + 4 < b.len() {
        if &b[i..i + 4] == b"\"id\"" {
            let mut j = i + 4;
            while j < b.len() && (b[j] == b':' || b[j] == b' ' || b[j] == b'\t') {
                j += 1;
            }
            let num_start = j;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            if j > num_start
                && let Ok(n) = before[num_start..j].parse::<u64>()
            {
                last_id = Some(n);
            }
            i = j;
        } else {
            i += 1;
        }
    }
    if last_id.is_some() {
        return last_id;
    }
    // Fallback: first "id" after the name in the window
    let after = &window[rel..];
    let mut i = 0;
    let b = after.as_bytes();
    while i + 4 < b.len() {
        if &b[i..i + 4] == b"\"id\"" {
            let mut j = i + 4;
            while j < b.len() && (b[j] == b':' || b[j] == b' ' || b[j] == b'\t') {
                j += 1;
            }
            let num_start = j;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            if j > num_start
                && let Ok(n) = after[num_start..j].parse::<u64>()
            {
                return Some(n);
            }
        }
        i += 1;
    }
    None
}

fn install_release(tag: &str, target: &str, prefix: &Path) -> Result<(), String> {
    let ext = if is_windows_target(target) {
        "zip"
    } else {
        "tar.gz"
    };
    let filename = format!("ods-{tag}-{target}.{ext}");

    eprintln!("ods: downloading {filename}…");

    let direct_archive_url = format!(
        "https://github.com/open-doc-spec/ods/releases/download/{tag}/{filename}"
    );
    let direct_sums_url = format!(
        "https://github.com/open-doc-spec/ods/releases/download/{tag}/SHA256SUMS"
    );

    let (archive, sums_bytes) = match (
        http_get_bytes(&direct_archive_url),
        http_get_bytes(&direct_sums_url),
    ) {
        (Ok(a), Ok(s)) => (a, s),
        _ => {
            let release_url = format!("{API_BASE}/releases/tags/{tag}");
            let release_json = http_get_string(&release_url)?;
            let archive_id = find_asset_id(&release_json, &filename).ok_or_else(|| {
                ods_core::error::update_asset_not_found(&filename, tag)
            })?;
            let sums_id = find_asset_id(&release_json, "SHA256SUMS").ok_or_else(|| {
                ods_core::error::update_checksums_not_found(tag)
            })?;
            let a = http_get_asset(archive_id)?;
            let s = http_get_asset(sums_id)?;
            (a, s)
        }
    };

    let sums = String::from_utf8_lossy(&sums_bytes);
    let expected = find_checksum(&sums, &filename)
        .ok_or_else(|| ods_core::error::update_checksum_entry_missing(&filename))?;
    let actual = hex_sha256(&archive);
    if actual != expected {
        return Err(ods_core::error::update_checksum_mismatch(
            &filename, &expected, &actual,
        ));
    }

    let tmp = env::temp_dir().join(format!(
        "ods-update-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&tmp)
        .map_err(|e| ods_core::error::detail("create update temp dir", e))?;
    let archive_path = tmp.join(&filename);
    fs::write(&archive_path, &archive)
        .map_err(|e| ods_core::error::detail("write update archive", e))?;

    let extract_dir = tmp.join("out");
    fs::create_dir_all(&extract_dir)
        .map_err(|e| ods_core::error::detail("create extract dir", e))?;

    if is_windows_target(target) {
        extract_zip(&archive_path, &extract_dir)?;
    } else {
        extract_tar_gz(&archive_path, &extract_dir)?;
    }

    let bin_src = find_ods_binary(&extract_dir, is_windows_target(target))?;
    fs::create_dir_all(prefix).map_err(|e| {
        ods_core::error::detail(&format!("create install dir {}", prefix.display()), e)
    })?;

    let ods_name = if is_windows_target(target) {
        "ods.exe"
    } else {
        "ods"
    };

    // Primary Open Document Spec CLI
    replace_binary(&bin_src, &prefix.join(ods_name))?;

    let _ = fs::remove_dir_all(&tmp);
    Ok(())
}

fn find_checksum(sums: &str, filename: &str) -> Option<String> {
    for line in sums.lines() {
        let line = line.trim();
        if line.ends_with(filename) {
            let hash = line.split_whitespace().next()?;
            return Some(hash.to_ascii_lowercase());
        }
    }
    None
}

fn hex_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod test_installer {
    use super::*;

    #[test]
    fn test_installer_helpers() {
        let json = r#"{"assets":[{"id":12345,"name":"ods-v0.1.0-linux-x86_64.tar.gz"},{"id":67890,"name":"SHA256SUMS"}]}"#;
        assert_eq!(find_asset_id(json, "ods-v0.1.0-linux-x86_64.tar.gz"), Some(12345));
        assert_eq!(find_asset_id(json, "SHA256SUMS"), Some(67890));

        let sums = "a1b2c3d4  ods-v0.1.0-linux-x86_64.tar.gz\n";
        assert_eq!(find_checksum(sums, "ods-v0.1.0-linux-x86_64.tar.gz"), Some("a1b2c3d4".to_string()));

        let hash = hex_sha256(b"hello");
        assert!(!hash.is_empty());

        let td = tempfile::tempdir().unwrap();
        assert!(install_release("invalid_tag", "x86_64-unknown-linux-gnu", td.path()).is_err());
    }
}

