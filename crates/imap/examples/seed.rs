#[path = "../tests/support/mod.rs"]
mod support;

use mail_sieve_imap::{Credentials, ImapConfig, Security};

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let usage = "usage: seed <host:port> <user> <count> [connections]";
    let target = args.next().expect(usage);
    let user = args.next().expect(usage);
    let count: usize = args
        .next()
        .expect(usage)
        .parse()
        .expect("count must be a number");
    let connections: usize = args.next().map_or(8, |value| {
        value.parse().expect("connections must be a number")
    });
    let (host, port) = target.rsplit_once(':').expect(usage);
    let config = ImapConfig {
        host: host.to_string(),
        port: port.parse().expect("port must be a number"),
        security: Security::Plain,
        username: user,
        credentials: Credentials::Password("pass".into()),
        compress: false,
    };
    let started = std::time::Instant::now();
    support::seed(&config, "INBOX", 0..count, connections).await;
    println!(
        "appended {count} messages in {:.1}s",
        started.elapsed().as_secs_f64()
    );
}
