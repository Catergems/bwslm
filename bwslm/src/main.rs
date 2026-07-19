mod distro;
mod wsl;
mod download;
mod verify;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bwslm", about = "Better WSL Manager")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a distro from the repo
    Add {
        distro: String,
        #[arg(long = "n")]
        name: Option<String>,
    },
    /// Import a distro from a URL or local file
    Import {
        #[arg(long, conflicts_with = "tar")]
        url: Option<String>,
        #[arg(long, conflicts_with = "url")]
        tar: Option<String>,
        #[arg(long = "n")]
        name: Option<String>,
    },
    /// Unregister and remove a distro
    Remove { distro: String },
    /// Shutdown distros (-a all, -d specific)
    Shutdown {
        #[arg(short = 'a', long = "all", conflicts_with = "distro")]
        all: bool,
        #[arg(short = 'd', long = "distro")]
        distro: Option<String>,
    },
    /// Launch, set default, or set default user for a distro
    Distro {
        distro: String,
        #[arg(short = 's', long = "set-default")]
        set_default: bool,
        #[arg(short = 'u', long = "user")]
        user: Option<String>,
    },
    /// Execute a command inside a distro
    Exec {
        distro: String,
        #[arg(last = true)]
        cmd: Vec<String>,
    },
    /// Clear download cache
    Prune,
    /// List installed WSL distros
    List,
    /// Repo management
    Repo {
        #[command(subcommand)]
        action: RepoAction,
    },
    /// Show version and build info
    Info,
}

#[derive(Subcommand)]
enum RepoAction {
    /// List distros available in the repo
    List,
    /// Update distro definitions from GitHub
    Update,
    /// Search for a distro in the repo
    Search {
        query: String,
        /// Show detailed info
        #[arg(long)]
        info: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result: anyhow::Result<()> = match cli.command {
        None => wsl::launch_default(),
        Some(Commands::Add { distro, name }) => wsl::install(&distro, name.as_deref()),
        Some(Commands::Import { url, tar, name }) => {
            let source = url.or(tar).unwrap_or_default();
            let n = name.unwrap_or_else(|| {
                std::path::Path::new(&source)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });
            wsl::import(&source, &n)
        }
        Some(Commands::Remove { distro }) => wsl::remove(&distro),
        Some(Commands::Shutdown { all, distro }) => {
            if all || distro.is_none() {
                wsl::shutdown_all()
            } else {
                wsl::shutdown_one(&distro.unwrap())
            }
        }
        Some(Commands::Distro { distro, set_default, user }) => {
            if set_default {
                wsl::set_default(&distro)
            } else if let Some(u) = user {
                wsl::set_default_user(&distro, &u)
            } else {
                wsl::launch_distro(&distro)
            }
        }
        Some(Commands::Exec { distro, cmd }) => wsl::exec(&distro, &cmd),
        Some(Commands::Prune) => wsl::prune(),
        Some(Commands::List) => wsl::list_installed(),
        Some(Commands::Repo { action }) => match action {
            RepoAction::List => distro::list_repo(),
            RepoAction::Update => distro::update_repo(),
            RepoAction::Search { query, info } => distro::search_repo(&query, info),
        },
        Some(Commands::Info) => info(),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn info() -> anyhow::Result<()> {
    println!("bwslm version : {}", read_version());
    println!("Better WSL Manager");
    match std::process::Command::new("wsl").arg("--version").output() {
        Ok(o) => print!("{}", String::from_utf8_lossy(&o.stdout).replace('\x00', "")),
        Err(_) => println!("WSL version: (could not retrieve)"),
    }
    Ok(())
}

pub fn read_version() -> String {
    let exe = std::env::current_exe().unwrap_or_default();
    let path = exe.parent().unwrap_or(std::path::Path::new(".")).join("version.txt");
    std::fs::read_to_string(path)
        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string())
        .trim()
        .to_string()
}
