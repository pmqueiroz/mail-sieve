#![allow(dead_code)]

use mail_sieve_imap::{Connection, Credentials, ImapConfig, Security, connect};

pub const SENDERS: &[(&str, &str, bool)] = &[
    ("Deals Daily", "deals@shop.example", true),
    ("Weekly News", "news@paper.example", true),
    ("Alice", "alice@friends.example", false),
    ("Build Bot", "ci@builds.example", true),
    ("Bob", "bob@friends.example", false),
];

pub fn server() -> Option<(String, u16)> {
    let target = std::env::var("MAIL_SIEVE_TEST_IMAP").ok()?;
    let (host, port) = target.rsplit_once(':')?;
    Some((host.to_string(), port.parse().ok()?))
}

pub fn config(user: &str) -> Option<ImapConfig> {
    let (host, port) = server()?;
    Some(ImapConfig {
        host,
        port,
        security: Security::Plain,
        username: user.to_string(),
        credentials: Credentials::Password("pass".into()),
        compress: true,
    })
}

pub fn unique_user(prefix: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{prefix}-{nanos}")
}

pub fn generate_message(index: usize) -> Vec<u8> {
    let (name, address, newsletter) = SENDERS[index % SENDERS.len()];
    let day = 1 + index % 28;
    let mut message = format!(
        "From: {name} <{address}>\r\nTo: me@example.com\r\nSubject: Message number {index}\r\n\
         Date: {day:02} Mar 2024 10:00:00 +0000\r\nMessage-ID: <{index}@mail-sieve.test>\r\n"
    );
    if newsletter {
        let domain = address.split('@').nth(1).unwrap();
        message.push_str(&format!(
            "List-Id: Updates <updates.{domain}>\r\nList-Unsubscribe: <mailto:unsub@{domain}>, <https://{domain}/u/{index}>\r\n\
             List-Unsubscribe-Post: List-Unsubscribe=One-Click\r\n"
        ));
    }
    message.push_str(
        "Content-Type: text/plain; charset=utf-8\r\n\r\nHello there, this is the body.\r\n",
    );
    message.into_bytes()
}

pub async fn seed(
    config: &ImapConfig,
    mailbox: &str,
    range: std::ops::Range<usize>,
    connections: usize,
) {
    let indexes: Vec<usize> = range.collect();
    let per_connection = indexes.len().div_ceil(connections.max(1)).max(1);
    let tasks = indexes.chunks(per_connection).map(|chunk| async move {
        let mut conn: Connection = connect(config).await.expect("connect for seeding");
        for &index in chunk {
            conn.session
                .append(mailbox, None, None, generate_message(index))
                .await
                .expect("append");
        }
        conn.logout().await;
    });
    futures::future::join_all(tasks).await;
}
