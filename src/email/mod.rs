use crate::config::types::EmailConfig;
use async_imap::Session;
use async_imap::types::Fetch;
use async_native_tls::TlsStream;
use futures::{StreamExt, TryStreamExt};
use mail_parser::{MessageParser, PartType};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::task::JoinHandle;
use tracing::{error, info};

#[derive(Debug)]
pub struct EmailService {
    email: String,
    password: String,
    server: String,
    port: u16,

    tx: Sender<IncomingEmail>,
}

#[derive(Debug, Clone)]
pub struct Address {
    pub name: Option<String>,
    pub adl: Option<String>,
    pub mailbox: Option<String>,
    pub host: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IncomingEmail {
    pub subject: Option<String>,
    pub from: Vec<Address>,
    pub body: Option<Vec<String>>,
}

fn utf8(opt: Option<&[u8]>) -> Option<String> {
    opt.and_then(|b| std::str::from_utf8(b).ok().map(String::from))
}

impl EmailService {
    pub fn new(config: &EmailConfig) -> Self {
        let (tx, _) = broadcast::channel(64);

        Self {
            email: config.email.clone(),
            password: config.password.clone(),
            server: config.server.clone(),
            port: config.port,
            tx,
        }
    }

    pub fn subscribe(&self) -> Receiver<IncomingEmail> {
        self.tx.subscribe()
    }

    fn get_subject(msg: &Fetch) -> Option<String> {
        msg.envelope()
            .and_then(|e| e.subject.as_ref())
            .and_then(|s| std::str::from_utf8(s).ok())
            .map(String::from)
    }

    fn get_body(msg: &Fetch) -> Option<Vec<String>> {
        let parsed = MessageParser::default().parse(msg.body()?)?;

        let res: Vec<String> = parsed
            .parts
            .iter()
            .filter_map(|part| match &part.body {
                PartType::Text(text) => Some(text.to_string()),
                _ => None,
            })
            .collect();

        if res.is_empty() { None } else { Some(res) }
    }

    fn get_sender(msg: &Fetch) -> Vec<Address> {
        msg.envelope()
            .and_then(|env| env.from.as_deref())
            .map(|list| {
                list.iter()
                    .map(|a| Address {
                        name: utf8(a.name.as_deref()),
                        adl: utf8(a.adl.as_deref()),
                        mailbox: utf8(a.mailbox.as_deref()),
                        host: utf8(a.host.as_deref()),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    async fn process_messages(
        imap_session: &mut async_imap::Session<async_native_tls::TlsStream<TcpStream>>,
        tx: &Sender<IncomingEmail>,
    ) -> anyhow::Result<()> {
        let _mailbox = imap_session.select("INBOX").await?;

        let uids = imap_session.uid_search("ALL").await?;
        let uid_set = uids.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(",");

        if uid_set.is_empty() {
            return Ok(());
        }

        let message_stream =
            imap_session.uid_fetch(&uid_set, "(FLAGS ENVELOPE BODY.PEEK[])").await?;
        let messages: Vec<Fetch> = message_stream.try_collect().await?;

        for msg in messages.iter() {
            if let Some(uid) = msg.uid {
                let subject = Self::get_subject(msg);
                let from = Self::get_sender(msg);
                let body = Self::get_body(msg);

                let email = IncomingEmail { subject, from, body };

                info!("New mail: {:?}", email);

                if let Err(e) = tx.send(email) {
                    error!("Failed to broadcast email (no receivers?): {e}");
                }

                imap_session.uid_mv(uid.to_string(), "Processed").await?;
            }
        }
        Ok(())
    }

    pub fn run_loop(self: Arc<Self>) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                match Self::run(&self).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("EmailService error: {e:?}, restarting in 5s...");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        })
    }

    pub async fn connect_and_login(&self) -> anyhow::Result<Session<TlsStream<TcpStream>>> {
        let imap_addr = (self.server.clone(), self.port);
        let tcp_stream = TcpStream::connect(imap_addr.clone()).await?;
        let tls = async_native_tls::TlsConnector::new();
        let tls_stream = tls.connect(self.server.clone(), tcp_stream).await?;

        let client = async_imap::Client::new(tls_stream);
        Ok(client.login(self.email.clone(), self.password.clone()).await.map_err(|e| e.0)?)
    }

    pub async fn ensure_processed_mailbox(
        imap_session: &mut Session<TlsStream<TcpStream>>,
    ) -> anyhow::Result<()> {
        let mut has_processed = false;
        {
            let mut mailboxes_stream = imap_session.list(None, Some("*")).await?;

            while let Some(result) = mailboxes_stream.next().await {
                match result {
                    Ok(name) => {
                        if name.name() == "Processed" {
                            has_processed = true;
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error listing mailboxes: {e}");
                    }
                }
            }
        }

        if !has_processed {
            let _ = imap_session.create("Processed").await;
        }

        Ok(())
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let mut imap_session = self.connect_and_login().await?;

        Self::ensure_processed_mailbox(&mut imap_session).await?;

        Self::process_messages(&mut imap_session, &self.tx).await?;

        loop {
            let mut idle = imap_session.idle();
            idle.init().await?;

            {
                let wait_future = idle.wait();
                match wait_future.0.await {
                    Ok(_) => {
                        info!("Wake-up from IDLE: fetching new messages");
                    }
                    Err(e) => {
                        error!("IDLE wait failed (connection likely lost): {e:?}");
                        break;
                    }
                }
            }

            imap_session = idle.done().await?;

            Self::process_messages(&mut imap_session, &self.tx).await?;
        }
        Ok(())
    }
}
