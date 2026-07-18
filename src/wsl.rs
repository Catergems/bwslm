use std::path::PathBuf;
use std::process::Command;

use crate::distro;
use crate::download;
use crate::verify;

fn install_root() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_default();
    exe.parent().unwrap_or(std::path::Path::new(".")).join("bwslos")
}

fn cache_dir() -> PathBuf {
    std::env::temp_dir().join("bwslm-cache")
}

pub fn launch_default() -> anyhow::Result<()> {
    Command::new("wsl").status()?;
    Ok(())
}

pub fn set_default(name: &str) -> anyhow::Result<()> {
    let status = Command::new("wsl").args(["--set-default", name]).status()?;
    if !status.success() {
        anyhow::bail!("wsl --set-default failed");
    }
    println!("Default distro set to {}.", name);
    Ok(())
}

pub fn exec(distro: &str, cmd: &[String]) -> anyhow::Result<()> {
    if cmd.is_empty() {
        anyhow::bail!("No command provided. Usage: bwslm exec <distro> -- <command>");
    }
    let status = Command::new("wsl")
        .arg("-d")
        .arg(distro)
        .arg("--")
        .args(cmd)
        .status()?;
    if !status.success() {
        anyhow::bail!("Command exited with status: {}", status);
    }
    Ok(())
}

pub fn install(name: &str, custom_name: Option<&str>) -> anyhow::Result<()> {
    let d = distro::find(name)?;
    let wsl_name = custom_name.unwrap_or(&d.name);
    let install_dir = install_root().join(wsl_name);

    let cache = cache_dir();
    std::fs::create_dir_all(&cache)?;

    let filename = std::path::Path::new(&d.url)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let local_file = cache.join(&filename);

    download::download(&d.url, &local_file)?;

    if d.checksum.is_some() || !d.sigs.is_empty() {
        verify::check(
            &local_file,
            d.checksum.as_deref(),
            d.checksumtype.as_deref(),
            &d.sigs,
        )?;
    }

    std::fs::create_dir_all(&install_dir)?;
    println!("Installing {} into {}...", wsl_name, install_dir.display());

    let status = Command::new("wsl")
        .args(["--import", wsl_name, &install_dir.to_string_lossy(), &local_file.to_string_lossy()])
        .status()?;

    if !status.success() {
        anyhow::bail!("wsl --import failed");
    }

    println!("{} installed successfully.", wsl_name);
    if !d.info.is_empty() {
        println!("Info: {}", d.info);
    }
    Ok(())
}

pub fn import(source: &str, name: &str) -> anyhow::Result<()> {
    let install_dir = install_root().join(name);
    std::fs::create_dir_all(&install_dir)?;

    let local_file = if source.starts_with("http://") || source.starts_with("https://") {
        let cache = cache_dir();
        std::fs::create_dir_all(&cache)?;
        let filename = std::path::Path::new(source)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let dest = cache.join(&filename);
        download::download(source, &dest)?;
        dest
    } else {
        PathBuf::from(source)
    };

    println!("Importing {} into {}...", name, install_dir.display());

    let status = Command::new("wsl")
        .args(["--import", name, &install_dir.to_string_lossy(), &local_file.to_string_lossy()])
        .status()?;

    if !status.success() {
        anyhow::bail!("wsl --import failed");
    }

    println!("{} imported successfully.", name);
    Ok(())
}

pub fn remove(name: &str) -> anyhow::Result<()> {
    println!("Unregistering {}...", name);
    let status = Command::new("wsl").args(["--unregister", name]).status()?;
    if !status.success() {
        anyhow::bail!("wsl --unregister failed");
    }

    let dir = install_root().join(name);
    if dir.exists() {
        println!("Removing {}...", dir.display());
        std::fs::remove_dir_all(&dir)?;
    }

    println!("{} removed.", name);
    Ok(())
}

pub fn shutdown_all() -> anyhow::Result<()> {
    println!("Shutting down all WSL distros...");
    let status = Command::new("wsl").arg("--shutdown").status()?;
    if !status.success() {
        anyhow::bail!("wsl --shutdown failed");
    }
    println!("Done.");
    Ok(())
}

pub fn shutdown_one(name: &str) -> anyhow::Result<()> {
    println!("Terminating {}...", name);
    let status = Command::new("wsl").args(["--terminate", name]).status()?;
    if !status.success() {
        anyhow::bail!("wsl --terminate {} failed", name);
    }
    println!("Done.");
    Ok(())
}

pub fn list_installed() -> anyhow::Result<()> {
    Command::new("wsl").args(["--list", "--verbose"]).status()?;
    Ok(())
}
