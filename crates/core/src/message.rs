use crate::Flags;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Address {
    pub name: Option<String>,
    pub email: String,
}

impl Address {
    pub fn domain(&self) -> &str {
        self.email.rsplit_once('@').map_or("", |(_, domain)| domain)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Unsubscribe {
    pub http: Option<String>,
    pub mailto: Option<String>,
    pub one_click: bool,
}

impl Unsubscribe {
    pub fn is_available(&self) -> bool {
        self.http.is_some() || self.mailto.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MessageMeta {
    pub uid: u32,
    pub message_id: Option<String>,
    pub gm_msgid: Option<u64>,
    pub from: Option<Address>,
    pub subject: Option<String>,
    pub date: Option<i64>,
    pub internal_date: Option<i64>,
    pub size: u32,
    pub flags: Flags,
    pub list_id: Option<String>,
    pub unsubscribe: Unsubscribe,
}
