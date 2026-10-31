mod support;

use mail_sieve_core::{Flags, UidSet};
use mail_sieve_imap::{
    HeaderFetchPlan, Operation, Provider, apply, connect, fetch_flags, fetch_headers_parallel,
    fetch_raw, list_folders, search_all_uids,
};
use tokio::sync::mpsc;

macro_rules! require_server {
    ($user:expr) => {
        match support::config(&support::unique_user($user)) {
            Some(config) => config,
            None => {
                eprintln!("MAIL_SIEVE_TEST_IMAP not set, skipping");
                return;
            }
        }
    };
}

#[tokio::test]
async fn lists_folders_with_special_use() {
    let config = require_server!("folders");
    let mut conn = connect(&config).await.unwrap();
    assert!(conn.capabilities.supports_move());
    assert!(conn.capabilities.condstore());
    assert_eq!(Provider::detect(&conn.capabilities), Provider::Generic);
    let folders = list_folders(&mut conn).await.unwrap();
    let special = |name: &str| {
        folders
            .iter()
            .find(|f| f.name == name)
            .and_then(|f| f.special_use.clone())
    };
    assert_eq!(special("Trash").as_deref(), Some("\\Trash"));
    assert_eq!(special("Archive").as_deref(), Some("\\Archive"));
    assert_eq!(special("INBOX"), None);
    conn.logout().await;
}

#[tokio::test]
async fn fetches_headers_over_parallel_connections() {
    let config = require_server!("parallel");
    support::seed(&config, "INBOX", 0..1200, 4).await;
    let mut conn = connect(&config).await.unwrap();
    let status = conn.examine("INBOX").await.unwrap();
    assert_eq!(status.exists, 1200);
    let uids = search_all_uids(&mut conn).await.unwrap();
    assert_eq!(uids.len(), 1200);

    let set = UidSet::from_uids(uids);
    let plan = HeaderFetchPlan {
        config: &config,
        mailbox: "INBOX",
        uidvalidity: status.uidvalidity,
        uids: &set,
        connections: 3,
        chunk_size: 250,
    };
    let (sender, mut receiver) = mpsc::channel(8);
    let collector = tokio::spawn(async move {
        let mut all = Vec::new();
        while let Some(batch) = receiver.recv().await {
            all.extend(batch);
        }
        all
    });
    fetch_headers_parallel(&plan, sender).await.unwrap();
    let messages = collector.await.unwrap();
    assert_eq!(messages.len(), 1200);
    let by_subject = |index: usize| {
        let subject = format!("Message number {index}");
        messages
            .iter()
            .find(|m| m.subject.as_deref() == Some(subject.as_str()))
            .unwrap()
    };
    let first = by_subject(0);
    assert_eq!(first.from.as_ref().unwrap().email, "deals@shop.example");
    assert_eq!(first.subject.as_deref(), Some("Message number 0"));
    assert_eq!(first.list_id.as_deref(), Some("updates.shop.example"));
    assert!(first.unsubscribe.one_click);
    assert!(first.size > 0);
    assert!(!by_subject(2).unsubscribe.is_available());

    let wrong = HeaderFetchPlan {
        uidvalidity: status.uidvalidity + 1,
        ..plan
    };
    let (sender, _receiver) = mpsc::channel(8);
    assert!(fetch_headers_parallel(&wrong, sender).await.is_err());
    conn.logout().await;
}

#[tokio::test]
async fn applies_bulk_operations() {
    let config = require_server!("actions");
    support::seed(&config, "INBOX", 0..40, 2).await;
    let mut conn = connect(&config).await.unwrap();
    let before = conn.select("INBOX").await.unwrap();
    let uids = search_all_uids(&mut conn).await.unwrap();

    apply(
        &mut conn,
        &UidSet::from_uids(uids[0..10].to_vec()),
        &Operation::AddFlags(Flags::SEEN),
    )
    .await
    .unwrap();
    let changed = fetch_flags(&mut conn, "1:*", before.highestmodseq)
        .await
        .unwrap();
    assert_eq!(changed.len(), 10);
    assert!(
        changed
            .iter()
            .all(|state| state.flags.contains(Flags::SEEN))
    );

    apply(
        &mut conn,
        &UidSet::from_uids(uids[0..5].to_vec()),
        &Operation::RemoveFlags(Flags::SEEN),
    )
    .await
    .unwrap();
    apply(
        &mut conn,
        &UidSet::from_uids(uids[0..5].to_vec()),
        &Operation::Move("Archive".into()),
    )
    .await
    .unwrap();
    apply(
        &mut conn,
        &UidSet::from_uids(uids[5..10].to_vec()),
        &Operation::Expunge,
    )
    .await
    .unwrap();
    assert_eq!(
        search_all_uids(&mut conn).await.unwrap(),
        uids[10..].to_vec()
    );

    let raw = fetch_raw(&mut conn, uids[10]).await.unwrap().unwrap();
    assert!(String::from_utf8_lossy(&raw).contains("this is the body"));
    let unread = fetch_flags(&mut conn, &uids[10].to_string(), None)
        .await
        .unwrap();
    assert!(!unread[0].flags.contains(Flags::SEEN));

    let archive = conn.examine("Archive").await.unwrap();
    assert_eq!(archive.exists, 5);
    conn.logout().await;
}
