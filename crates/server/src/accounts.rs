use mail_sieve_imap::{
    Connection, Credentials, ImapConfig, Provider, Security, connect, list_folders,
};
use mail_sieve_store::accounts::{Account, NewAccount};
use mail_sieve_store::mailboxes::ListedMailbox;
use mail_sieve_unsub::{SmtpAuth, SmtpSettings};

use crate::error::{Error, Result};
use crate::oauth::{self, OAuthProvider};
use crate::state::AppState;
use crate::vault::StoredCredentials;

const TOKEN_REFRESH_MARGIN: i64 = 120;

async fn secret(state: &AppState, account: &Account) -> Result<(Credentials, SmtpAuth)> {
    match state.vault.open(&account.credentials)? {
        StoredCredentials::Password { password } => Ok((
            Credentials::Password(password.clone()),
            SmtpAuth::Password(password),
        )),
        StoredCredentials::OAuth2 {
            provider,
            refresh_token,
            access_token,
            expires_at,
        } => {
            let now = mail_sieve_store::now();
            if let (Some(token), Some(expires)) = (&access_token, expires_at)
                && expires - TOKEN_REFRESH_MARGIN > now
            {
                return Ok((
                    Credentials::OAuth2 {
                        access_token: token.clone(),
                    },
                    SmtpAuth::OAuth2(token.clone()),
                ));
            }
            let oauth_provider = OAuthProvider::parse(&provider)
                .ok_or_else(|| Error::OAuth(format!("unknown provider {provider}")))?;
            let tokens = oauth::refresh(state, oauth_provider, &refresh_token).await?;
            let refreshed = StoredCredentials::OAuth2 {
                provider,
                refresh_token: tokens.refresh_token.unwrap_or(refresh_token),
                access_token: Some(tokens.access_token.clone()),
                expires_at: Some(now + tokens.expires_in.unwrap_or(3600)),
            };
            let sealed = state.vault.seal(&refreshed)?;
            let account_id = account.id;
            state
                .write(move |tx| {
                    mail_sieve_store::accounts::update_credentials(tx, account_id, &sealed)
                })
                .await?;
            Ok((
                Credentials::OAuth2 {
                    access_token: tokens.access_token.clone(),
                },
                SmtpAuth::OAuth2(tokens.access_token),
            ))
        }
    }
}

pub async fn imap_config(state: &AppState, account: &Account) -> Result<ImapConfig> {
    let (credentials, _) = secret(state, account).await?;
    Ok(ImapConfig {
        host: account.imap_host.clone(),
        port: account.imap_port,
        security: if account.imap_tls {
            Security::Tls
        } else {
            Security::Plain
        },
        username: account.username.clone(),
        credentials,
        compress: true,
    })
}

pub async fn smtp_settings(state: &AppState, account: &Account) -> Result<Option<SmtpSettings>> {
    let Some(host) = account.smtp_host.clone() else {
        return Ok(None);
    };
    let (_, auth) = secret(state, account).await?;
    Ok(Some(SmtpSettings {
        host,
        port: account.smtp_port.unwrap_or(587),
        username: account.username.clone(),
        auth,
        from: account.email.clone(),
    }))
}

pub async fn load(state: &AppState, account_id: i64) -> Result<Account> {
    state
        .read(move |conn| mail_sieve_store::accounts::get(conn, account_id))
        .await
}

pub async fn refresh_folders(
    state: &AppState,
    account_id: i64,
    conn: &mut Connection,
) -> Result<Provider> {
    let provider = Provider::detect(&conn.capabilities);
    let folders = list_folders(conn).await?;
    let listed: Vec<ListedMailbox> = folders
        .iter()
        .map(|folder| ListedMailbox {
            name: folder.name.clone(),
            delimiter: folder.delimiter.clone(),
            special_use: folder.special_use.clone(),
            default_sync: provider.default_sync(folder),
        })
        .collect();
    let capabilities = conn.capabilities.names().join(" ");
    let provider_name = provider.to_string();
    state
        .write(move |tx| {
            mail_sieve_store::accounts::set_server_info(
                tx,
                account_id,
                &provider_name,
                &capabilities,
            )?;
            mail_sieve_store::mailboxes::sync_listing(tx, account_id, &listed)
        })
        .await?;
    Ok(provider)
}

pub struct AccountInput {
    pub label: Option<String>,
    pub email: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub imap_tls: bool,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub username: String,
    pub auth_kind: &'static str,
    pub oauth_provider: Option<String>,
    pub credentials: StoredCredentials,
}

pub async fn create(state: &AppState, input: AccountInput) -> Result<Account> {
    let sealed = state.vault.seal(&input.credentials)?;
    let draft = Account {
        id: 0,
        label: input.label.clone().unwrap_or_else(|| input.email.clone()),
        email: input.email.clone(),
        imap_host: input.imap_host.clone(),
        imap_port: input.imap_port,
        imap_tls: input.imap_tls,
        smtp_host: input.smtp_host.clone(),
        smtp_port: input.smtp_port,
        username: input.username.clone(),
        auth_kind: input.auth_kind.to_string(),
        oauth_provider: input.oauth_provider.clone(),
        credentials: sealed.clone(),
        provider: "generic".into(),
        max_connections: 4,
        capabilities: String::new(),
        sync_interval_minutes: 15,
        created_at: 0,
        last_synced_at: None,
        last_error: None,
    };
    let config = imap_config(state, &draft).await?;
    let mut conn = connect(&config).await?;
    let provider = Provider::detect(&conn.capabilities);
    let new_account = NewAccount {
        label: draft.label.clone(),
        email: draft.email.clone(),
        imap_host: draft.imap_host.clone(),
        imap_port: draft.imap_port,
        imap_tls: draft.imap_tls,
        smtp_host: draft.smtp_host.clone(),
        smtp_port: draft.smtp_port,
        username: draft.username.clone(),
        auth_kind: draft.auth_kind.clone(),
        oauth_provider: draft.oauth_provider.clone(),
        credentials: sealed,
        provider: provider.to_string(),
        max_connections: provider.default_connections(),
    };
    let account_id = state
        .write(move |tx| mail_sieve_store::accounts::insert(tx, &new_account))
        .await?;
    let refreshed = refresh_folders(state, account_id, &mut conn).await;
    conn.logout().await;
    if let Err(error) = refreshed {
        state
            .write(move |tx| mail_sieve_store::accounts::delete(tx, account_id))
            .await?;
        return Err(error);
    }
    load(state, account_id).await
}
