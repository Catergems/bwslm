use std::io::{self, Write};
use std::time::Instant;

const BAR_WIDTH: usize = 50;
const SPINNER: &[&str] = &["|", "/", "-", "\\"];

pub fn download(url: &str, dest: &std::path::Path) -> anyhow::Result<()> {
    let client = reqwest::blocking::Client::new();
    let mut resp = client.get(url).send()?;

    if !resp.status().is_success() {
        anyhow::bail!("Download failed: HTTP {}", resp.status());
    }

    let total = resp.content_length();
    let filename = std::path::Path::new(url)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    if let Some(t) = total {
        println!("Downloading {} ({:.1} MB)", filename, t as f64 / 1_000_000.0);
    } else {
        println!("Downloading {}", filename);
    }

    let mut file = std::fs::File::create(dest)?;
    let mut downloaded: u64 = 0;
    let start = Instant::now();
    let mut spin_idx = 0usize;
    let mut buf = vec![0u8; 8192];

    loop {
        use std::io::Read;
        let n = resp.read(&mut buf)?;
        if n == 0 { break; }
        file.write_all(&buf[..n])?;
        downloaded += n as u64;

        let pct = total.map(|t| downloaded as f64 / t as f64).unwrap_or(0.0);
        let filled = (pct * BAR_WIDTH as f64) as usize;
        let bar: String = "■".repeat(filled) + &" ".repeat(BAR_WIDTH - filled);
        let spin = SPINNER[spin_idx % SPINNER.len()];
        spin_idx += 1;

        let elapsed = start.elapsed().as_secs_f64();
        let est = if pct > 0.01 && pct < 1.0 {
            format!("EST: {:.0}s", (elapsed / pct) - elapsed)
        } else {
            "EST: --".to_string()
        };

        print!("\r[{}] {:.0}%  {}  {}   ", bar, pct * 100.0, spin, est);
        io::stdout().flush()?;
    }

    let elapsed = start.elapsed().as_secs_f64();
    let bar = "■".repeat(BAR_WIDTH);
    println!("\r[{}] 100%  -  EST: 0s          ", bar);
    let _ = elapsed;

    Ok(())
}
