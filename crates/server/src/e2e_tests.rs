use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use mail_sieve_imap::{Credentials, ImapConfig, Security, connect, search_all_uids};
use mail_sieve_store::Store;
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::config::Config;
use crate::state::AppState;
use crate::vault::Vault;
use crate::{api, jobs};

const SENDERS: &[(&str, bool)] = &[
    ("deals@shop.example", true),
    ("news@paper.example", true),
    ("alice@friends.example", false),
];

fn server() -> Option<(String, u16)> {
    let target = std::env::var("MAIL_SIEVE_TEST_IMAP").ok()?;
    let (host, port) = target.rsplit_once(':')?;
    Some((host.to_string(), port.parse().ok()?))
}

fn message(index: usize) -> String {
    let (address, newsletter) = SENDERS[index % SENDERS.len()];
    let mut text = format!(
        "From: Sender <{address}>\r\nSubject: Message {index}\r\nDate: 01 Mar 2024 10:00:00 +0000\r\nMessage-ID: <{index}@e2e.test>\r\n"
    );
    if newsletter {
        text.push_str("List-Unsubscribe: <https://unsubscribe.example/u>\r\n");
    }
    text.push_str("\r\nBody of message number ");
    text.push_str(&index.to_string());
    text.push_str("\r\n");
    text
}

async fn seed(config: &ImapConfig, range: std::ops::Range<usize>) {
    let mut conn = connect(config).await.unwrap();
    for index in range {
        conn.session
            .append("INBOX", None, None, message(index))
            .await
            .unwrap();
    }
    conn.logout().await;
}

struct Harness {
    router: Router,
    cookie: Option<String>,
}

impl Harness {
    async fn call(&self, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
        let mut request = Request::builder().method(method).uri(uri);
        if let Some(cookie) = &self.cookie {
            request = request.header(header::COOKIE, cookie);
        }
        let request = match body {
            Some(body) => request
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string())),
            None => request.body(Body::empty()),
        }
        .unwrap();
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, value)
    }

    async fn login_cookie(&mut self, path: &str, password: &str) -> StatusCode {
        let request = Request::post(path)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({ "password": password }).to_string()))
            .unwrap();
        let response = self.router.clone().oneshot(request).await.unwrap();
        if let Some(cookie) = response.headers().get(header::SET_COOKIE) {
            let cookie = cookie.to_str().unwrap();
            assert!(cookie.contains("HttpOnly") && cookie.contains("SameSite=Lax"));
            self.cookie = Some(cookie.split(';').next().unwrap().to_string());
        }
        response.status()
    }

    async fn wait_for_job(&self, job_id: i64) -> Value {
        for _ in 0..300 {
            let (_, job) = self.call("GET", &format!("/api/jobs/{job_id}"), None).await;
            if matches!(job["state"].as_str(), Some("done" | "failed" | "cancelled")) {
                return job;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        panic!("job {job_id} did not finish");
    }

    async fn count(&self, account: i64, query: &str) -> u64 {
        let encoded: String = query
            .replace(' ', "%20")
            .replace('@', "%40")
            .replace(':', "%3A");
        let (status, totals) = self
            .call(
                "GET",
                &format!("/api/accounts/{account}/count?q={encoded}"),
                None,
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{totals}");
        totals["count"].as_u64().unwrap()
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn full_flow_against_dovecot() {
    let Some((host, port)) = server() else {
        eprintln!("MAIL_SIEVE_TEST_IMAP not set, skipping");
        return;
    };
    let user = format!(
        "e2e-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let imap = ImapConfig {
        host: host.clone(),
        port,
        security: Security::Plain,
        username: user.clone(),
        credentials: Credentials::Password("pass".into()),
        compress: true,
    };
    seed(&imap, 0..300).await;

    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        data_dir: dir.path().to_path_buf(),
        bind: "127.0.0.1:0".parse().unwrap(),
        public_url: None,
        secret_key: None,
        admin_password: None,
        trusted_header: None,
        google_client_id: None,
        google_client_secret: None,
        microsoft_client_id: None,
        microsoft_client_secret: None,
    };
    let key = config.load_secret_key().unwrap();
    let store = Store::open(config.database_path()).unwrap();
    let state = AppState::new(config, store, Vault::new(&key));
    jobs::spawn_runner(state.clone());
    let mut harness = Harness {
        router: api::router(state.clone()),
        cookie: None,
    };

    let (status, _) = harness.call("GET", "/api/accounts", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        harness.login_cookie("/api/auth/setup", "short").await,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        harness
            .login_cookie("/api/auth/setup", "a long enough password")
            .await,
        StatusCode::OK
    );
    assert_eq!(
        harness
            .login_cookie("/api/auth/setup", "another long password")
            .await,
        StatusCode::CONFLICT
    );
    harness.cookie = None;
    assert_eq!(
        harness
            .login_cookie("/api/auth/login", "wrong password!")
            .await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        harness
            .login_cookie("/api/auth/login", "a long enough password")
            .await,
        StatusCode::OK
    );

    let (status, account) = harness
        .call(
            "POST",
            "/api/accounts",
            Some(json!({ "email": user, "imap_host": host, "imap_port": port, "security": "plain", "password": "pass" })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{account}");
    assert!(account.get("credentials").is_none());
    let account_id = account["id"].as_i64().unwrap();

    let (_, job_list) = harness.call("GET", "/api/jobs", None).await;
    let sync_job = job_list
        .as_array()
        .unwrap()
        .iter()
        .find(|job| job["kind"] == "sync")
        .unwrap()["id"]
        .as_i64()
        .unwrap();
    let job = harness.wait_for_job(sync_job).await;
    assert_eq!(job["state"], "done", "{job}");
    assert_eq!(job["done"], 300);
    assert_eq!(harness.count(account_id, "").await, 300);

    let (_, groups) = harness
        .call(
            "GET",
            &format!("/api/accounts/{account_id}/groups?sort=count"),
            None,
        )
        .await;
    let groups = groups.as_array().unwrap();
    assert_eq!(groups.len(), 3);
    assert!(groups.iter().all(|group| group["message_count"] == 100));
    let (_, domains) = harness
        .call(
            "GET",
            &format!("/api/accounts/{account_id}/groups?group=domain&q=has%3Aunsubscribe"),
            None,
        )
        .await;
    assert_eq!(domains.as_array().unwrap().len(), 2);

    let (status, _) = harness
        .call(
            "POST",
            &format!("/api/accounts/{account_id}/actions"),
            Some(json!({ "query": "", "action": "archive" })),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = harness
        .call(
            "POST",
            &format!("/api/accounts/{account_id}/actions"),
            Some(json!({ "query": "from:alice", "action": "delete" })),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, response) = harness
        .call(
            "POST",
            &format!("/api/accounts/{account_id}/actions"),
            Some(json!({ "query": "from:deals@shop.example", "action": "archive" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{response}");
    let job = harness
        .wait_for_job(response["job_id"].as_i64().unwrap())
        .await;
    assert_eq!(job["state"], "done", "{job}");
    assert_eq!(job["done"], 100);
    assert_eq!(
        harness.count(account_id, "from:deals@shop.example").await,
        0
    );

    let mut conn = connect(&imap).await.unwrap();
    conn.examine("INBOX").await.unwrap();
    assert_eq!(search_all_uids(&mut conn).await.unwrap().len(), 200);
    assert_eq!(conn.examine("Archive").await.unwrap().exists, 100);

    let (_, page) = harness
        .call(
            "GET",
            &format!("/api/accounts/{account_id}/messages?q=from%3Aalice&limit=5"),
            None,
        )
        .await;
    let messages = page["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 5);
    assert!(page["next"].is_object());
    let ids: Vec<i64> = messages.iter().map(|m| m["id"].as_i64().unwrap()).collect();
    let (_, response) = harness
        .call(
            "POST",
            &format!("/api/accounts/{account_id}/actions"),
            Some(json!({ "ids": ids, "action": "mark_read" })),
        )
        .await;
    harness
        .wait_for_job(response["job_id"].as_i64().unwrap())
        .await;
    assert_eq!(harness.count(account_id, "is:read").await, 5);

    let (status, preview) = harness
        .call("GET", &format!("/api/messages/{}/preview", ids[0]), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{preview}");
    assert!(
        preview["text"]
            .as_str()
            .unwrap()
            .contains("Body of message number")
    );

    seed(&imap, 300..310).await;
    conn.select("INBOX").await.unwrap();
    let uids = search_all_uids(&mut conn).await.unwrap();
    mail_sieve_imap::apply(
        &mut conn,
        &mail_sieve_core::UidSet::from_uids(uids[..3].to_vec()),
        &mail_sieve_imap::Operation::Expunge,
    )
    .await
    .unwrap();
    conn.logout().await;
    let (_, response) = harness
        .call("POST", &format!("/api/accounts/{account_id}/sync"), None)
        .await;
    let job = harness
        .wait_for_job(response["job_id"].as_i64().unwrap())
        .await;
    assert_eq!(job["state"], "done", "{job}");
    assert_eq!(harness.count(account_id, "in:inbox").await, 207);
    assert_eq!(harness.count(account_id, "is:read").await, 5);

    let (status, _) = harness.call("POST", "/api/auth/logout", None).await;
    assert_eq!(status, StatusCode::OK);
}
