mod accounts;
mod actions;
mod api;
mod assets;
mod auth;
mod config;
#[cfg(test)]
mod e2e_tests;
mod error;
mod idle;
mod jobs;
mod oauth;
mod rules;
mod scheduler;
mod state;
mod sync;
mod undo;
mod vault;

use std::io::BufRead;

use clap::{Parser, Subcommand};
use mail_sieve_store::Store;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::state::AppState;
use crate::vault::Vault;

#[derive(Parser)]
#[command(
    name = "mail-sieve",
    version,
    about = "Self-hosted IMAP mailbox control"
)]
struct Cli {
    #[command(flatten)]
    config: Config,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Serve,
    Sync {
        #[arg(long)]
        account: i64,
    },
    Accounts,
    SetPassword,
    Health,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stdout()))
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=warn")),
        )
        .init();
    let cli = Cli::parse();
    if let Err(error) = run(cli).await {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

async fn open_state(config: Config) -> Result<AppState> {
    std::fs::create_dir_all(&config.data_dir)
        .map_err(|e| Error::Internal(format!("{}: {e}", config.data_dir.display())))?;
    let key = config.load_secret_key()?;
    let store = Store::open(config.database_path())?;
    Ok(AppState::new(config, store, Vault::new(&key)))
}

async fn run(cli: Cli) -> Result<()> {
    if let Some(Command::Health) = cli.command {
        return health(&cli.config).await;
    }
    let state = open_state(cli.config).await?;
    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => serve(state).await,
        Command::Sync { account } => {
            let report = sync::sync_account(&state, account, None).await?;
            let seconds = report.elapsed.as_secs_f64();
            println!(
                "synced {} mailboxes: {} new, {} removed, {} flag updates in {:.2}s ({:.0} messages/s)",
                report.mailboxes,
                report.fetched,
                report.removed,
                report.flags_updated,
                seconds,
                report.fetched as f64 / seconds.max(0.001)
            );
            Ok(())
        }
        Command::Accounts => {
            for account in state.read(mail_sieve_store::accounts::list).await? {
                println!(
                    "{}\t{}\t{}:{}\t{}\tlast sync: {}",
                    account.id,
                    account.email,
                    account.imap_host,
                    account.imap_port,
                    account.provider,
                    account
                        .last_synced_at
                        .map_or("never".to_string(), |at| at.to_string())
                );
            }
            Ok(())
        }
        Command::Health => Ok(()),
        Command::SetPassword => {
            eprintln!("new admin password:");
            let mut password = String::new();
            std::io::stdin()
                .lock()
                .read_line(&mut password)
                .map_err(|e| Error::Internal(e.to_string()))?;
            set_admin_password(&state, password.trim_end_matches(['\r', '\n'])).await?;
            println!("admin password updated; existing sessions were signed out");
            Ok(())
        }
    }
}

async fn health(config: &Config) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut address = config.bind;
    if address.ip().is_unspecified() {
        address.set_ip(std::net::Ipv4Addr::LOCALHOST.into());
    }
    let unhealthy = |reason: String| Error::Internal(format!("unhealthy: {reason}"));
    let mut stream = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        tokio::net::TcpStream::connect(address),
    )
    .await
    .map_err(|_| unhealthy("timeout".into()))?
    .map_err(|e| unhealthy(e.to_string()))?;
    stream
        .write_all(b"GET /api/health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .map_err(|e| unhealthy(e.to_string()))?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .await
        .map_err(|e| unhealthy(e.to_string()))?;
    if response.starts_with("HTTP/1.1 200") {
        Ok(())
    } else {
        Err(unhealthy(response.lines().next().unwrap_or("").into()))
    }
}

async fn set_admin_password(state: &AppState, password: &str) -> Result<()> {
    if password.chars().count() < auth::MIN_PASSWORD_LENGTH {
        return Err(Error::BadRequest(format!(
            "password must have at least {} characters",
            auth::MIN_PASSWORD_LENGTH
        )));
    }
    let password = password.to_string();
    let hash = tokio::task::spawn_blocking(move || auth::hash_password(&password)).await??;
    state
        .write(move |tx| {
            mail_sieve_store::settings::set(
                tx,
                mail_sieve_store::settings::ADMIN_PASSWORD_HASH,
                &hash,
            )?;
            mail_sieve_store::sessions::delete_all(tx)
        })
        .await
}

async fn serve(state: AppState) -> Result<()> {
    if let Some(password) = state.config.admin_password.clone() {
        let current = state
            .read(|c| {
                mail_sieve_store::settings::get(c, mail_sieve_store::settings::ADMIN_PASSWORD_HASH)
            })
            .await?;
        let matches = match current {
            Some(hash) => {
                tokio::task::spawn_blocking({
                    let password = password.clone();
                    move || auth::verify_password(&password, &hash)
                })
                .await?
            }
            None => false,
        };
        if !matches {
            set_admin_password(&state, &password).await?;
        }
    }
    jobs::spawn_runner(state.clone());
    scheduler::spawn(state.clone());
    let listener = tokio::net::TcpListener::bind(state.config.bind)
        .await
        .map_err(|e| Error::Internal(format!("cannot bind {}: {e}", state.config.bind)))?;
    tracing::info!(address = %state.config.bind, "mail-sieve listening");
    axum::serve(listener, api::router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| Error::Internal(e.to_string()))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            signal.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}
