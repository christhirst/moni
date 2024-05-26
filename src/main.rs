use std::error::Error;
//use std::time::Duration;
use config::Config;
use ldap3::result::LdapResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::io::{AsyncWriteExt, Interest};
use tokio::net::TcpStream;
use tokio::time::{self, timeout, Duration};

#[derive(Debug)]
pub enum ConnectionError {
    ConnectionRefused,
    HostNotKnown,
}

// /ticker
struct Status {
    hosts: String,
    http_status: Result<bool, ConnectionError>,
    ldap_status: Result<LdapResult, ConnectionError>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub debug: bool,
    pub key: String,
    pub hosts: Vec<Host>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Host {
    pub authority: String,
    pub scheme: String,
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
async fn tcp_checker(authority: &str, config_timeout: u64) -> Result<TcpStream, ConnectionError> {
    let timeout_duration = Duration::from_secs(config_timeout);
    let result = timeout(timeout_duration, TcpStream::connect(authority)).await;
    let result = match result {
        Ok(result) => match result {
            Ok(ok) => Ok(ok),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("Connection refused") {
                    Err(ConnectionError::ConnectionRefused)
                } else if err_str.contains("Host not known") {
                    Err(ConnectionError::HostNotKnown)
                } else {
                    panic!("{}", format!("{}", e))
                }
            }
        },
        Err(_) => todo!(),
    };

    result
}

async fn ldap_checker() -> Result<LdapResult, ConnectionError> {
    todo!()
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
    let conf = settings.try_deserialize::<Settings>().unwrap();
    println!("{:?}", conf.clone());

    let mut interval = time::interval(Duration::from_secs(3));
    loop {
        let status = tcp_checker(conf.hosts[0].authority.as_str(), conf.hosts[0].interval).await;

        interval.tick().await;
        println!("{status:?} - tick");
    }

    Ok(())
}
