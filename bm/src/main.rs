use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bm", about = "bwslm updater")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Update bwslm to the latest release
    Update,
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Update => update(),
    };
    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn update() -> anyhow::Result<()> {
    let script_url = "https://raw.githubusercontent.com/Catergems/bwslm/main/update.ps1";
    let script_path = std::env::temp_dir().join("bwslm-update.ps1");

    println!("Downloading update script...");
    let client = reqwest::blocking::Client::builder()
        .user_agent("bm-updater")
        .build()?;
    let content = client.get(script_url).send()?.text()?;
    std::fs::write(&script_path, content)?;

    println!("Launching updater...");
    std::process::Command::new("powershell")
        .args([
            "-ExecutionPolicy", "Bypass",
            "-File", &script_path.to_string_lossy(),
        ])
        .spawn()?;

    println!("Update started. bm will now exit.");
    Ok(())
}
