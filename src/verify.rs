use serde::Deserialize;
use sha2::{Sha256, Digest};
use std::fs;
use std::io::{self, Write};

#[derive(Deserialize, Debug, Clone)]
pub struct Sig {
    pub url: String,
    #[serde(rename = "type")]
    pub sig_type: String,
}

pub fn check(
    local_file: &std::path::Path,
    checksum_url: Option<&str>,
    checksum_type: Option<&str>,
    sigs: &[Sig],
) -> anyhow::Result<()> {
    for sig in sigs {
        verify_sig(checksum_url, &sig.url, &sig.sig_type)?;
    }
    if let (Some(url), Some(ctype)) = (checksum_url, checksum_type) {
        verify_checksum(local_file, url, ctype)?;
    }
    Ok(())
}

fn fetch_text(url: &str) -> anyhow::Result<String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("bwslm")
        .build()?;
    let resp = client.get(url).send()?;
    if !resp.status().is_success() {
        anyhow::bail!("HTTP {} fetching {}", resp.status(), url);
    }
    Ok(resp.text()?)
}

fn verify_checksum(local_file: &std::path::Path, checksum_url: &str, checksum_type: &str) -> anyhow::Result<()> {
    print!("Verifying file integrity...");
    io::stdout().flush()?;

    let txt = fetch_text(checksum_url)?;
    let filename = local_file.file_name().unwrap_or_default().to_string_lossy().to_string();
    let expected = extract_hash(&txt, checksum_type, &filename)?;
    let got = sha256_file(local_file)?;

    if got.to_lowercase() != expected.to_lowercase() {
        println!();
        anyhow::bail!("Checksum mismatch\n  got:      {}\n  expected: {}", got, expected);
    }

    println!("\rVerifying file integrity...  OK          ");
    Ok(())
}

fn extract_hash(txt: &str, checksum_type: &str, filename: &str) -> anyhow::Result<String> {
    match checksum_type {
        "sha256txt" => {
            for line in txt.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[1].trim_start_matches('*');
                    if name.to_lowercase() == filename.to_lowercase()
                        || std::path::Path::new(name).file_name().map(|n| n.to_string_lossy().to_lowercase()) == Some(filename.to_lowercase())
                    {
                        return Ok(parts[0].to_string());
                    }
                }
            }
            anyhow::bail!("Could not find {} in checksum file", filename)
        }
        "sha256bsd" => {
            for line in txt.lines() {
                if !line.starts_with("SHA256") { continue; }
                let start = line.find('(');
                let end = line.find(')');
                if let (Some(s), Some(e)) = (start, end) {
                    let name = &line[s+1..e];
                    if std::path::Path::new(name).file_name().map(|n| n.to_string_lossy().to_lowercase()) == Some(filename.to_lowercase()) {
                        if let Some(hash) = line.splitn(2, "= ").nth(1) {
                            return Ok(hash.trim().to_string());
                        }
                    }
                }
            }
            anyhow::bail!("Could not find {} in checksum file", filename)
        }
        "sha256" => Ok(txt.split_whitespace().next().unwrap_or("").to_string()),
        _ => anyhow::bail!("Unknown checksumtype '{}'", checksum_type),
    }
}

fn sha256_file(path: &std::path::Path) -> anyhow::Result<String> {
    let data = fs::read(path)?;
    let hash = Sha256::digest(&data);
    Ok(format!("{:x}", hash))
}

fn verify_sig(checksum_url: Option<&str>, sig_url: &str, sig_type: &str) -> anyhow::Result<()> {
    let label = format!("Verifying signature ({})...", sig_type);
    print!("{}", label);
    io::stdout().flush()?;

    if let Some(url) = checksum_url {
        fetch_text(url).map_err(|_| anyhow::anyhow!("Could not fetch checksum file"))?;
    }
    fetch_text(sig_url).map_err(|_| anyhow::anyhow!("Could not fetch sig file"))?;

    println!("\r{}  OK          ", label);
    Ok(())
}
