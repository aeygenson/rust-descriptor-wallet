use crate::WalletSyncResult;

/// Abstraction for broadcasting a fully signed raw transaction to the network.
///
/// `wallet_sync` defines only the behavior contract. Concrete implementations
/// (Esplora, Bitcoin Core RPC, mocks, etc.) live in backend modules.
///
/// Implementations should:
/// - return `Ok(())` only when the transaction is accepted for broadcast
/// - return structured `WalletSyncError::*` variants for failure cases
/// - avoid panicking on network or parsing errors
pub trait TxBroadcaster {
    /// Broadcast a raw transaction serialized as hex.
    ///
    /// The transaction must be fully signed and valid.
    fn broadcast_tx_hex(&self, tx_hex: &str) -> WalletSyncResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::core_rpc::broadcast::CoreRpcBroadcaster;
    use crate::backend::esplora::broadcast::EsploraBroadcaster;
    use crate::backend::mock::broadcast::{FailingBroadcaster, NoopBroadcaster};
    use crate::WalletSyncError;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn noop_broadcaster_succeeds() {
        let broadcaster = NoopBroadcaster;
        let result = broadcaster.broadcast_tx_hex("deadbeef");

        assert!(result.is_ok());
    }

    #[test]
    fn failing_broadcaster_returns_transport_error() {
        let broadcaster = FailingBroadcaster::new("mock broadcast failure");
        let result = broadcaster.broadcast_tx_hex("deadbeef");

        assert!(matches!(
            result,
            Err(WalletSyncError::BroadcastTransport(_))
        ));
    }

    #[test]
    fn failing_broadcaster_can_return_structured_transport_error() {
        let broadcaster = FailingBroadcaster::transport("network down");
        let result = broadcaster.broadcast_tx_hex("deadbeef");

        assert!(matches!(
            result,
            Err(WalletSyncError::BroadcastTransport(msg)) if msg.contains("network down")
        ));
    }

    #[test]
    fn retries_and_succeeds_on_retryable_503_error() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test listener");
        let addr = listener.local_addr().expect("listener addr");

        let handle = thread::spawn(move || {
            for attempt in 0..2 {
                let (mut stream, _) = listener.accept().expect("accept connection");

                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf).expect("read request");

                if attempt == 0 {
                    let body = "temporary server error";
                    let response = format!(
                        "HTTP/1.1 503 Service Unavailable\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    stream
                        .write_all(response.as_bytes())
                        .expect("write 503 response");
                } else {
                    let response =
                        "HTTP/1.1 200 OK\r\ncontent-length: 0\r\nconnection: close\r\n\r\n";
                    stream
                        .write_all(response.as_bytes())
                        .expect("write 200 response");
                }
                stream.flush().expect("flush response");
            }
        });

        let broadcaster = EsploraBroadcaster::new(format!("http://{}", addr));
        let result = broadcaster.broadcast_tx_hex("deadbeef");

        assert!(result.is_ok());
        handle.join().expect("server thread join");
    }

    #[test]
    fn core_rpc_broadcaster_succeeds_on_json_rpc_result() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test listener");
        let addr = listener.local_addr().expect("listener addr");

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept connection");

            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf).expect("read request");

            let body = r#"{"result":"deadbeef-txid","error":null,"id":"wallet_sync"}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .expect("write 200 response");
            stream.flush().expect("flush response");
        });

        let broadcaster = CoreRpcBroadcaster::new(format!("http://{}", addr), "bitcoin", "bitcoin");
        let result = broadcaster.broadcast_tx_hex("deadbeef");

        assert!(result.is_ok());
        handle.join().expect("server thread join");
    }
}
