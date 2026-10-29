#![allow(clippy::all)]
use ns::diffengine::{calculate_changes, gather_hashes};
use std::{collections::HashMap, env::args, path::PathBuf, thread::sleep, time::Duration};
use tokio::{net::UdpSocket, time::Interval};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let args: Vec<String> = args().collect();
    let serve = match args[1].as_str() {
        "serve" => true,
        "get" => false,
        _ => false,
    };

    if serve {
        let socket = UdpSocket::bind("0.0.0.0:8080").await.unwrap();
        socket.set_broadcast(true).unwrap();
        let uuid = Uuid::new_v4();
        let mut buf = [0u8; 4096];
        let mut x = tokio::time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                _ = x.tick() => { handle_tick(&socket, uuid).await },
                result = socket.recv_from(&mut buf) => {
                    match result {
                        Ok((_len, peer_addr)) => {
                            // println!("Packet from: {}", peer_addr);
                        },
                        Err(e) => eprintln!("err: {e}"),
                    }
                },
            }
        }
    } else {
        let socket = UdpSocket::bind("0.0.0.0:8080").await.unwrap();
        let mut buf = [0u8; 4096];
        let mut known_hosts = HashMap::new();
        loop {
            let (len, addr) = socket.recv_from(&mut buf).await.unwrap();
            println!(
                "Received packet: {} from {}:{}",
                String::from_utf8_lossy(&buf[..len]),
                addr.ip(),
                addr.port()
            );
            let buf_clone = buf.clone();
            let msg = String::from_utf8_lossy(&buf_clone[..len]).to_string();
            if msg.trim().ends_with("hosting ns") {
                known_hosts.insert(addr.ip(), msg.split(" ").nth(0).unwrap().to_string());
            }
            for (ip, id) in &known_hosts {
                println!("Known host: {} at {}", id, ip);
            }
        }
    }
}

// let args: Vec<String> = args().collect();
// let path = PathBuf::from(&args[1]);
//
// let mut last_hashes = gather_hashes(path.clone());
// loop {
//     let hash = gather_hashes(path.clone());
//     if hash != last_hashes {
//         let diffs = calculate_changes(&last_hashes, &hash);
//         for x in diffs.modified {
//             println!("Modified: {}", x.display());
//         }
//         for x in diffs.new {
//             println!("New: {}", x.display());
//         }
//         for x in diffs.deleted {
//             println!("Deleted: {}", x.display());
//         }
//     }
//     sleep(Duration::from_secs(1));
//     last_hashes = hash;
// }
//
async fn handle_tick(socket: &UdpSocket, uuid: Uuid) {
    let msg = format!("{} hosting ns", uuid);
    // temp, change to 255.255.255.255 later
    let _ = socket.send_to(msg.as_bytes(), "255.255.255.255:8080").await;
    println!("Pinged");
}
