use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::thread;
//use std::time::Duration;
use config::Config;
use ldap3::result::LdapResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::io::{AsyncWriteExt, Interest};
use tokio::net::TcpStream;
use tokio::time::{self, timeout, Duration};
mod ldap;
pub mod tcp;

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
//#[serde(rename_all = "camelCase")]
pub struct Host {
    pub authority: String,
    pub bind_dn: String,
    pub bind_pw: String,
    pub base: String,
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
fn tcp_checker<'a>(
    authority: &'a str,
    config_timeout: &'a u64,
) -> impl Future<Output = Result<TcpStream, ConnectionError>> + 'a {
    async move {
        let timeout_duration = Duration::from_secs(*config_timeout);

        /* let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut result: Result<Result<TcpStream, std::io::Error>, time::error::Elapsed> =
            Ok(Ok(TcpStream::connect(socket_addr))); */

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
}

async fn ldap_checker() -> Result<LdapResult, ConnectionError> {
    todo!()
}
use std::future::Future;

async fn loop_spawn<'a, F, Fut>(
    h: &'a Host,
    //f: &dyn Fn() -> Result<TcpStream, ConnectionError>,
    f: F,
) where
    F: Fn(&'a str, &'a u64) -> Fut,
    Fut: Future<Output = Result<TcpStream, ConnectionError>> + Send,
{
    let mut interval = time::interval(Duration::from_secs(3));

    loop {
        let status = f(h.authority.as_str(), &h.interval).await;

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

    for i in conf.hosts {
        println!("ticks");
        // Spin up another thread
        tokio::spawn(async move { loop_spawn(&i, tcp::tcp_checker).await });
    }

    loop {}

    Ok(())
}
