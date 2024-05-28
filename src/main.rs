use std::collections::HashMap;
use std::error::Error;

//use std::time::Duration;
use config::Config;
use ldap3::result::LdapResult;
use serde::{Deserialize, Serialize};
use std::future::Future;
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{self, Duration};
mod ldap;
pub mod tcp;
use http::StatusCode;

#[derive(Debug, Clone)]
pub enum ConnectionError {
    ConnectionRefused,
    HostNotKnown,
}

// /ticker
#[derive(Default, Debug, Clone)]
struct Status {
    http_status: Vec<Result<StatusCode, ConnectionError>>,
    //ldap_status: Result<LdapResult, ConnectionError>,
}

#[derive(Default, Debug, Clone)]
struct gatherdstatus {
    status: Vec<Status>,
}
impl gatherdstatus {
    fn new() -> Self {
        Default::default()
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub debug: bool,
    pub key: String,
    pub hosts: Vec<Host>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
//#[serde(rename_all = "camelCase")]
pub struct Host {
    pub authority: String,
    pub bind_dn: String,
    pub bind_pw: String,
    pub base: String,
    pub filter: String,
    pub scheme: Option<String>,
    pub interval: u64,
}

/* #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    root: Root,
}
 */
/* #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub debug: bool,
    pub priority: i64,
    pub key: String,
}
 */

async fn loop_spawn<'a, F, Fut>(
    tx: UnboundedSender<HashMap<String, gatherdstatus>>,

    h: &'a Host,
    //f: &dyn Fn() -> Result<TcpStream, ConnectionError>,
    f: F,
) where
    F: Fn(&'a str, &'a u64) -> Fut,
    Fut: Future<Output = Result<StatusCode, ConnectionError>> + Send,
{
    let mut interval = time::interval(Duration::from_secs(3));
    let mut statusmap: HashMap<String, gatherdstatus> = HashMap::new();
    statusmap.insert(h.authority.clone(), gatherdstatus::new());
    loop {
        let status = f(h.authority.as_str(), &h.interval).await;

        tx.send(statusmap.clone());

        interval.tick().await;
        println!("{status:?} - tick");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings = Config::builder()
        // Add in `./Settings.toml`
        .add_source(config::File::with_name("Config.toml"))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();

    // Print out our settings (as a HashMap)
    let conf: Settings = settings.try_deserialize::<Settings>().unwrap();
    println!("{conf:?}");

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    for i in conf.hosts {
        let tx_clone = tx.clone();
        println!("ticks");
        // Spin up another thread
        tokio::spawn(async move { loop_spawn(tx_clone, &i, tcp::tcp_checker).await });
    }

    loop {
        let received = rx.recv().await.unwrap();
        println!("{received:?}");

        for message in &received {
            println!("{message:?}");
        }
    }

    Ok(())
}
