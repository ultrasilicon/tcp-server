use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use anyhow::Result;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast,
};

use broadcast::error::RecvError::*;

use usize as ClientId;

const ADDR: &str = "0.0.0.0:8888";
const BROADCAST_CHANNEL_SIZE: usize = 1000;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let listener = TcpListener::bind(ADDR).await?;
    println!("async listening on port {ADDR}");

    let (tx, _) = broadcast::channel::<(ClientId, String)>(BROADCAST_CHANNEL_SIZE);
    let id_counter = Arc::new(AtomicUsize::new(0));

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        println!("connected {} {}", peer_addr.ip(), peer_addr.port());

        let tx_rx = tx.clone();
        let mut rx = tx.subscribe();
        let client_id = id_counter.fetch_add(1, Ordering::Relaxed);

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, client_id, tx_rx, &mut rx).await {
                eprintln!("client {} error: {}", peer_addr.port(), e);
            }
            println!("client {} disconnected", peer_addr.port());
        });
    }
}

async fn handle_client(
    socket: TcpStream,
    client_id: ClientId,
    tx: broadcast::Sender<(ClientId, String)>,
    rx: &mut broadcast::Receiver<(ClientId, String)>,
) -> Result<()> {
    let (reader, mut writer) = socket.into_split();
    let mut lines = BufReader::new(reader).lines();

    // send login msg
    writer
        .write_all(format!("LOGIN:{client_id}\n").as_bytes())
        .await?;
    writer.flush().await?;

    loop {
        tokio::select! {
            // read from client
            result = lines.next_line() => {
                match result {
                    Ok(Some(msg)) => {
                        eprintln!("message {}: {}", writer.peer_addr()?.port(), msg);

                        // send ack msg
                        writer
                            .write_all(b"ACK:MESSAGE\n")
                            .await?;
                        writer.flush().await?;

                        // send received msg to broadcast
                        let _ = tx.send((client_id, msg));
                    }
                    Ok(None) => break, // handle client close
                    Err(e) => {
                        eprintln!("read error from {client_id}: {e}");
                        break;
                    }
                }
            }

            // receive broadcast msg & write to client
            result = rx.recv() => {
                match result {
                    Ok((sender_id, msg)) => if sender_id != client_id {
                        writer
                            .write_all(format!("MESSAGE:{sender_id} {msg}\n").as_bytes())
                            .await?;
                    },
                    Err(Lagged(n)) => {
                        eprintln!("client {client_id} lagged {n} messages");
                    },
                    Err(Closed) => break,
                }
            }
        }
    }

    Ok(())
}
