use crate::ConnectionError;
use crate::Status;
use http::StatusCode;
use std::{future::Future, time::Duration};
use tokio::{net::TcpStream, time::timeout};

pub async fn tcp_checker<'a>(
    authority: &'a str,
    config_timeout: &'a u64,
) -> Result<StatusCode, ConnectionError> {
    //async move {
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

    //TODO move to own function

    let result = match result {
        Ok(ok) => Ok(http::status::StatusCode::OK),
        Err(err) => Ok(http::status::StatusCode::SERVICE_UNAVAILABLE),
    };

    //result
    // }
    result
}
