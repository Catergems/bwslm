use colored::Colorize;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::verify::Sig;

#[derive(Deserialize, Debug)]
pub struct Distro {
    pub verjson: String,
    pub name: String,
    pub url: String,
    pub installationtype: String,
    #[serde(default)]
    pub info: String,
    #[serde(default)]
    pub checksum: Option<String>,
    #[serde(default)]
    pub checksumtype: Option<String>,
    #[serde(default)]
    pub sigs: Vec<Sig>,
}

pub fn distros_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        let p = exe.parent().unwrap_or(std::path::Path::new(".")).join("distros");
        if p.exists() { return p; }
    }
    std::env::current_dir().unwrap_or_default().join("distros")
}

pub fn load_all() -> anyhow::Result<Vec<Distro>> {
    let dir = distros_dir();
    let mut list = Vec::new();

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") { continue; }
        let data = fs::read_to_string(&path)?;
        match serde_json::from_str::<Distro>(&data) {
            Ok(d) => list.push(d),
            Err(e) => eprintln!("Warning: failed to parse {:?}: {e}", path.file_name()),
        }
    }

    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(list)
}

pub fn find(name: &str) -> anyhow::Result<Distro> {
    let list = load_all()?;
    list.into_iter()
        .find(|d| d.name.to_lowercase() == name.to_lowercase())
        .ok_or_else(|| anyhow::anyhow!("Distro '{}' not found in repo. Run 'bwslm repo list' to see available.", name))
}

fn installed_distros() -> Vec<String> {
    let out = Command::new("wsl").args(["--list", "--quiet"]).output();
    match out {
        Ok(o) => {
            String::from_utf8_lossy(&o.stdout)
                .replace('\x00', "")
                .lines()
                .map(|l| l.trim().to_lowercase())
                .filter(|l| !l.is_empty())
                .collect()
        }
        Err(_) => vec![],
    }
}

pub fn list_repo() -> anyhow::Result<()> {
    let list = load_all()?;
    if list.is_empty() {
        println!("No distros found in repo.");
        return Ok(());
    }

    let installed = installed_distros();

    println!();
    println!("  {}", "✦ bwslm repo ✦".bright_magenta().bold());
    println!();

    let mut installed_count = 0;
    for d in &list {
        let is_installed = installed.contains(&d.name.to_lowercase());
        if is_installed { installed_count += 1; }

        let marker = if is_installed {
            "✔".green().bold()
        } else {
            "·".dimmed().into()
        };

        let name = if is_installed {
            d.name.cyan().bold()
        } else {
            d.name.cyan().into()
        };

        let version = d.verjson.yellow();

        println!("  {} {:<22}  {}", marker, name, version);
    }

    println!();
    println!(
        "  {} · {}",
        format!("{} distros", list.len()).dimmed(),
        format!("{} installed", installed_count).green()
    );
    println!();

    Ok(())
}

pub fn update_repo() -> anyhow::Result<()> {
    println!("Updating distro definitions from GitHub...");

    let url = "https://api.github.com/repos/Catergems/bwslm/contents/distros";
    let client = reqwest::blocking::Client::builder()
        .user_agent("bwslm")
        .build()?;

    let resp = client.get(url).send()?;
    if !resp.status().is_success() {
        anyhow::bail!("Failed to fetch repo contents: HTTP {}", resp.status());
    }

    let files: Vec<GhFile> = resp.json()?;
    let dir = distros_dir();
    fs::create_dir_all(&dir)?;

    let mut count = 0;
    for file in files.iter().filter(|f| f.name.ends_with(".json")) {
        let content = client.get(&file.download_url).send()?.text()?;
        fs::write(dir.join(&file.name), content)?;
        println!("  Updated {}", file.name);
        count += 1;
    }

    println!("Done. Updated {} distro definition(s).", count);
    Ok(())
}

#[derive(Deserialize)]
struct GhFile {
    name: String,
    download_url: String,
}
