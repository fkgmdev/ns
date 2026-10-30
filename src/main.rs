#![allow(clippy::all)]
use std::net::SocketAddr;
use std::{collections::HashMap, env::args, io::stdin, net::IpAddr};
use tokio::{
    net::UdpSocket,
    time::{Duration, Instant, timeout_at},
};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let args: Vec<String> = args().collect();
    let serve = match args[1].as_str() {
        "serve" => true,
        "get" => false,
        _ => false,
    };
    // let folder = args[2].clone();

    if serve {
        server().await;
    } else {
        client().await;
    }
}

async fn server() {
    let socket = UdpSocket::bind("0.0.0.0:8080").await.unwrap();
    socket.set_broadcast(true).unwrap();
    let uuid = Uuid::new_v4();
    let mut buf = [0u8; 4096];
    let mut broadcast = tokio::time::interval(Duration::from_secs(1));
    let mut connect_checker = tokio::time::interval(Duration::from_secs(5));
    let mut connected = HashMap::new();
    loop {
        tokio::select! {
            _ = broadcast.tick() => { broadcast_msg(&socket, uuid).await },
            result = socket.recv_from(&mut buf) => {
                match result {
                    Ok((len, peer_addr)) => {
                        let buf_clone = buf.clone();
                        let msg = String::from_utf8_lossy(&buf_clone[..len]);
                        if msg.trim().ends_with("connect") {
                            let reply = "ok";
                            if let Ok(_) = socket.send_to(&reply.as_bytes(), peer_addr).await {
                                let peer_uuid = msg.split(" ").nth(0).unwrap().to_string();
                                connected.insert(peer_addr, (peer_uuid.clone(), Instant::now()));
                                println!("{} @ {} connected", peer_uuid, peer_addr);
                            }
                        } else if msg.trim().ends_with("check") {
                            if let Some((_, last_seen)) = connected.get_mut(&peer_addr) {
                                *last_seen = Instant::now();
                            }
                        }
                    },
                    Err(e) => eprintln!("err: {e}"),
                }
            },
            _ = connect_checker.tick() => {
                // connected = check_connect(&connected, &socket).await;
                send_check(&connected, &socket).await;
                let now = Instant::now();
                connected.retain(|_, (_, last_seen)| {
                    now.duration_since(*last_seen) < Duration::from_secs(12)
                });
                for (ip, (id, last_seen)) in &connected {
                    println!("Connected device: {} @ {} last seen {}s ago", id, ip, last_seen.elapsed().as_secs());
                }
            },
        }
    }
}

async fn client() {
    let socket = UdpSocket::bind("0.0.0.0:8080").await.unwrap();
    // let mut buf = [0u8; 4096];
    let mut known_hosts = HashMap::new();
    let uuid = Uuid::new_v4();

    // interface
    loop {
        for (index, (ip, id)) in known_hosts.iter().enumerate() {
            println!("({index}): {id} @ {ip}");
        }
        println!("S to scan, number to connect");
        let mut read_buffer = String::new();
        let _ = stdin().read_line(&mut read_buffer);
        if read_buffer.trim() == "s" {
            known_hosts = scan_for_servers(&socket).await;
        } else {
            let number: usize = read_buffer.trim().parse().unwrap();
            let (ip, _) = known_hosts.iter().nth(number).unwrap();
            match connect(&socket, uuid, ip).await {
                Ok(_) => {
                    println!("Connected");
                    break;
                }
                Err(e) => {
                    println!("Failed connect: {e}");
                }
            }
        }
    }
    let mut buf = [0u8; 4096];
    loop {
        tokio::select! {
           result = socket.recv_from(&mut buf) => {
                match result {
                    Ok((len, addr)) => {
                        let msg = String::from_utf8_lossy(&buf[..len]);
                        if msg.trim() == "check" {
                            let reply = "check";
                            let _ = socket.send_to(&reply.as_bytes(), addr).await;
                        }
                    },
                    Err(e) => {
                        eprintln!("Error: {e}");
                        continue;
                    },
                }
           }
        }
    }
}

async fn send_check(connected: &HashMap<SocketAddr, (String, Instant)>, socket: &UdpSocket) {
    if connected.is_empty() {
        return;
    }
    for (ip, _) in connected {
        let _ = socket.send_to(b"check", *ip).await;
    }
}

async fn check_connect(
    connected: &HashMap<SocketAddr, String>,
    socket: &UdpSocket,
) -> HashMap<SocketAddr, String> {
    if connected.is_empty() {
        return HashMap::new();
    }
    let mut result = HashMap::new();
    for (ip, id) in connected {
        let msg = "check";
        let _ = socket.send_to(&msg.as_bytes(), ip).await;
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut buf = [0u8; 1024];
        while let Ok(Ok((_len, addr))) = timeout_at(deadline, socket.recv_from(&mut buf)).await {
            if *ip == addr {
                result.insert(*ip, id.to_string());
            }
        }
    }
    result
}

async fn connect(socket: &UdpSocket, uuid: Uuid, ip: &IpAddr) -> Result<(), String> {
    let msg = format!("{uuid} connect");
    socket
        .send_to(msg.as_bytes(), (*ip, 8080))
        .await
        .map_err(|e| e.to_string())?;
    let mut buf = [0u8; 4096];
    let deadline = Instant::now() + Duration::from_secs(2);

    while let Ok(Ok((len, addr))) = timeout_at(deadline, socket.recv_from(&mut buf)).await {
        if addr.ip() == *ip && &buf[..len] == b"ok" {
            return Ok(());
        }
    }
    Err("Unspecified".to_string())
}

async fn scan_for_servers(socket: &UdpSocket) -> HashMap<IpAddr, String> {
    let mut buf = [0u8; 4096];
    let mut known_hosts = HashMap::new();
    let deadline = Instant::now() + Duration::from_secs(2);
    while let Ok(Ok((len, addr))) = timeout_at(deadline, socket.recv_from(&mut buf)).await {
        let buf_clone = buf.clone();
        let msg = String::from_utf8_lossy(&buf_clone[..len]).to_string();
        if msg.trim().ends_with("hosting ns") {
            known_hosts.insert(addr.ip(), msg.split(" ").nth(0).unwrap().to_string());
        }
    }
    known_hosts
}

async fn broadcast_msg(socket: &UdpSocket, uuid: Uuid) {
    let msg = format!("{} hosting ns", uuid);
    // temp, change to 255.255.255.255 later
    let _ = socket.send_to(msg.as_bytes(), "255.255.255.255:8080").await;
    println!("Pinged");
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
