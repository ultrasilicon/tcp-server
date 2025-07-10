use std::{
    collections::VecDeque,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use usize as ClientId;

struct Client {
    id: usize,
    stream: TcpStream,
    read_buf: Vec<u8>,
}

fn main() -> std::io::Result<()> {
    let addr = "0.0.0.0:8888";
    let listener = TcpListener::bind(addr)?;
    listener.set_nonblocking(true)?;  // we handle it async with our own runtime
    println!("listening on {}", addr);

    let mut clients = Vec::<Client>::new();
    let mut next_id: ClientId = 0;

    loop {
        // handle incoming connections
        if let Ok((mut stream, peer)) = listener.accept() {
            stream.set_nonblocking(true)?;
            let id = next_id;
            next_id += 1;
            println!("connected {}:{}", peer.ip(), peer.port());

            // send login msg
            let _ = stream.write_all(format!("LOGIN:{}\n", id).as_bytes());

            clients.push(Client {
                id,
                stream,
                read_buf: Vec::new(),
            });
        }

        // read from clients
        let mut to_remove = VecDeque::new();
        let mut broadcasts: Vec<(ClientId, String)> = Vec::new();

        for idx in 0..clients.len() {
            let client = &mut clients[idx];
            let mut buf = [0u8; 1024];
            match client.stream.read(&mut buf) {
                Ok(0) => {
                    // client disconnected
                    to_remove.push_back(idx);
                },
                Ok(n) => {
                    client.read_buf.extend_from_slice(&buf[..n]);
                    // read until first newline
                    while let Some(pos) = client.read_buf.iter().position(|&b| b == b'\n') {
                        let line = client.read_buf.drain(..=pos).collect::<Vec<u8>>();
                        let msg = String::from_utf8_lossy(&line).to_string();
                        print!("message {}: {}", client.stream.peer_addr().unwrap().port(), msg);

                        // send ack msg
                        let _ = client.stream.write_all(b"ACK:MESSAGE\n");

                        // queue for broadcasting
                        broadcasts.push((client.id, msg));
                    }
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}, // no data to read
                Err(_) => {
                    to_remove.push_back(idx);
                },
            }
        }

        // remove disconnected clients
        while let Some(idx) = to_remove.pop_back() {
            let client = clients.swap_remove(idx);
            println!("client {} disconnected", client.stream.peer_addr().unwrap().port());
        }

        // broadcast messages to all clients except the senders
        for (sender_id, msg) in broadcasts {
            let out = format!("MESSAGE:{} {}\n", sender_id, msg);
            for client in clients.iter_mut() {
                if client.id != sender_id {
                    let _ = client.stream.write_all(out.as_bytes());
                }
            }
        }

        // comment this line out if you don't pay electric bill
        thread::sleep(Duration::from_millis(10));
    }
}
