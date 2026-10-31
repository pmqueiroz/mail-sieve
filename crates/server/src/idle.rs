use std::time::Duration;

use mail_sieve_imap::{IdleResponse, connect};
use tokio::task::JoinHandle;

use crate::accounts;
use crate::error::Result;
use crate::jobs;
use crate::state::AppState;

const IDLE_TIMEOUT: Duration = Duration::from_secs(25 * 60);
const RETRY_DELAY: Duration = Duration::from_secs(120);
const DEBOUNCE: Duration = Duration::from_secs(5);

pub fn spawn_watcher(state: AppState, account_id: i64) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match watch(&state, account_id).await {
                Ok(false) => return,
                Ok(true) => {}
                Err(error) => tracing::debug!(%error, account_id, "idle watcher reconnecting"),
            }
            tokio::time::sleep(RETRY_DELAY).await;
        }
    })
}

async fn watch(state: &AppState, account_id: i64) -> Result<bool> {
    let account = accounts::load(state, account_id).await?;
    let mailboxes = state
        .read(move |c| mail_sieve_store::mailboxes::list(c, account_id))
        .await?;
    let target = mailboxes
        .iter()
        .find(|mailbox| mailbox.sync_enabled && mailbox.special_use.as_deref() == Some("\\All"))
        .or_else(|| {
            mailboxes
                .iter()
                .find(|mailbox| mailbox.sync_enabled && mailbox.name.eq_ignore_ascii_case("INBOX"))
        })
        .map(|mailbox| mailbox.name.clone());
    let Some(target) = target else {
        return Ok(false);
    };
    let config = accounts::imap_config(state, &account).await?;
    let mut conn = connect(&config).await?;
    if !conn.capabilities.idle() {
        conn.logout().await;
        return Ok(false);
    }
    conn.examine(&target).await?;
    let mut session = conn.session;
    loop {
        let mut handle = session.idle();
        handle.init().await.map_err(mail_sieve_imap::Error::from)?;
        let (wait, _stop) = handle.wait_with_timeout(IDLE_TIMEOUT);
        let response = wait.await.map_err(mail_sieve_imap::Error::from)?;
        session = handle.done().await.map_err(mail_sieve_imap::Error::from)?;
        if let IdleResponse::NewData(_) = response {
            tokio::time::sleep(DEBOUNCE).await;
            jobs::enqueue_sync(state, account_id).await?;
        }
    }
}
