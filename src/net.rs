use std::net::{SocketAddr, UdpSocket};

use ggrs::{PlayerType, SessionBuilder, UdpNonBlockingSocket};

use crate::FPS;
use crate::game::ActiveGame;
use crate::maps::Map;
use crate::simulation::GgrsConfig;

const CHUNK_SIZE: usize = 508;

pub enum HostPoll {
    WaitingForClient,
    SendingMap { total: usize },
    Ready(Box<ActiveGame>),
    Failed(String),
}

pub struct HostSyncer {
    socket: Option<UdpSocket>,
    pub port: u16,
    map: Map,
    map_bytes: Vec<u8>,
    total_chunks: u8,
    client_addr: Option<SocketAddr>,
    elapsed: f32,
    last_send: f32,
}

impl HostSyncer {
    pub fn new(port_str: &str, map_file: &str) -> Result<Self, String> {
        let port: u16 = port_str.parse().map_err(|_| "Invalid port".to_string())?;
        let map = Map::load(map_file);
        let map_data = map.to_string_repr();
        let map_bytes = map_data.into_bytes();
        let total_chunks = ((map_bytes.len() + CHUNK_SIZE - 1) / CHUNK_SIZE).max(1) as u8;

        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port))
            .map_err(|e| format!("Port is busy: {:?}", e))?;
        socket
            .set_nonblocking(true)
            .map_err(|e| format!("Nonblocking error: {:?}", e))?;

        Ok(Self {
            socket: Some(socket),
            port,
            map,
            map_bytes,
            total_chunks,
            client_addr: None,
            elapsed: 0.0,
            last_send: 0.0,
        })
    }

    pub fn update(&mut self, dt: f32) -> HostPoll {
        self.elapsed += dt;
        if self.elapsed > 25.0 {
            return HostPoll::Failed("Conection timeout (25s)".to_string());
        }

        let socket = match self.socket.as_ref() {
            Some(s) => s,
            None => return HostPoll::Failed("Socket closed".to_string()),
        };

        let mut buf = [0u8; 1024];

        while let Ok((len, src)) = socket.recv_from(&mut buf) {
            if &buf[..len] == b"REQ_MAP" {
                self.client_addr = Some(src);
            } else if let Some(target) = self.client_addr {
                if src == target && &buf[..len] == b"MAP_OK" {
                    self.socket = None;

                    return match start_host_session(self.port, target, self.map.clone()) {
                        Ok(game) => HostPoll::Ready(Box::new(game)),
                        Err(e) => HostPoll::Failed(e),
                    };
                }
            }
        }

        if let Some(target) = self.client_addr {
            if self.elapsed - self.last_send >= 0.08 {
                self.last_send = self.elapsed;
                for i in 0..self.total_chunks {
                    let start = i as usize * CHUNK_SIZE;
                    let end = (start + CHUNK_SIZE).min(self.map_bytes.len());
                    let slice = &self.map_bytes[start..end];

                    let mut packet = Vec::with_capacity(5 + slice.len());
                    packet.extend_from_slice(b"MAP");
                    packet.push(i);
                    packet.push(self.total_chunks);
                    packet.extend_from_slice(slice);

                    let _ = socket.send_to(&packet, target);
                }
            }
            HostPoll::SendingMap {
                total: self.total_chunks as usize,
            }
        } else {
            HostPoll::WaitingForClient
        }
    }
}

pub enum JoinPoll {
    Connecting,
    ReceivingMap { received: usize, total: usize },
    Ready(Box<ActiveGame>),
    Failed(String),
}

pub struct JoinSyncer {
    socket: Option<UdpSocket>,
    pub local_port: u16,
    pub host_addr: SocketAddr,
    chunks: Vec<Option<Vec<u8>>>,
    total_chunks: Option<u8>,
    elapsed: f32,
    last_req: f32,
}

impl JoinSyncer {
    pub fn new(
        local_port_str: &str,
        host_addr_str: &str,
        host_port_str: &str,
    ) -> Result<Self, String> {
        let local_port: u16 = local_port_str
            .parse()
            .map_err(|_| "Invalid local port".to_string())?;
        let host_port: u16 = host_port_str
            .parse()
            .map_err(|_| "Invalid host port".to_string())?;
        let host_addr: SocketAddr = format!("{}:{}", host_addr_str, host_port)
            .parse()
            .map_err(|_| "Invalid IP ot port".to_string())?;

        let socket = UdpSocket::bind(format!("0.0.0.0:{}", local_port))
            .map_err(|e| format!("Local port is busy: {:?}", e))?;
        socket
            .set_nonblocking(true)
            .map_err(|e| format!("Nonblocking error: {:?}", e))?;

        Ok(Self {
            socket: Some(socket),
            local_port,
            host_addr,
            chunks: Vec::new(),
            total_chunks: None,
            elapsed: 0.0,
            last_req: -1.0,
        })
    }

    pub fn update(&mut self, dt: f32) -> JoinPoll {
        self.elapsed += dt;
        if self.elapsed > 15.0 {
            return JoinPoll::Failed("No responce from host (15s)".to_string());
        }

        let socket = match self.socket.as_ref() {
            Some(s) => s,
            None => return JoinPoll::Failed("Socket closed".to_string()),
        };

        if self.elapsed - self.last_req >= 0.12 {
            self.last_req = self.elapsed;
            let _ = socket.send_to(b"REQ_MAP", self.host_addr);
        }

        let mut buf = [0u8; 1024];
        while let Ok((len, src)) = socket.recv_from(&mut buf) {
            if src == self.host_addr && len >= 5 && &buf[..3] == b"MAP" {
                let chunk_idx = buf[3] as usize;
                let count = buf[4];

                if self.total_chunks.is_none() {
                    self.total_chunks = Some(count);
                    self.chunks = vec![None; count as usize];
                }

                if chunk_idx < self.chunks.len() {
                    self.chunks[chunk_idx] = Some(buf[5..len].to_vec());
                }

                if self.chunks.iter().all(|c| c.is_some()) {
                    let mut full_bytes = Vec::new();
                    for part in &self.chunks {
                        full_bytes.extend_from_slice(part.as_ref().unwrap());
                    }

                    if let Ok(content) = String::from_utf8(full_bytes) {
                        if let Some(map) = Map::from_string_repr(&content) {
                            for _ in 0..4 {
                                let _ = socket.send_to(b"MAP_OK", self.host_addr);
                            }

                            self.socket = None;

                            return match start_join_session(self.local_port, self.host_addr, map) {
                                Ok(game) => JoinPoll::Ready(Box::new(game)),
                                Err(e) => JoinPoll::Failed(e),
                            };
                        }
                    }
                }
            }
        }

        let received = self.chunks.iter().filter(|c| c.is_some()).count();
        if let Some(total) = self.total_chunks {
            JoinPoll::ReceivingMap {
                received,
                total: total as usize,
            }
        } else {
            JoinPoll::Connecting
        }
    }
}

fn start_host_session(port: u16, client_addr: SocketAddr, map: Map) -> Result<ActiveGame, String> {
    let socket = UdpNonBlockingSocket::bind_to_port(port)
        .map_err(|e| format!("GGRS Bind Error: {:?}", e))?;

    let session = SessionBuilder::<GgrsConfig>::new()
        .with_num_players(2)
        .unwrap()
        .with_fps(FPS as usize)
        .unwrap()
        .with_input_delay(1)
        .add_player(PlayerType::Local, 0)
        .unwrap()
        .add_player(PlayerType::Remote(client_addr), 1)
        .unwrap()
        .start_p2p_session(socket)
        .unwrap();

    Ok(ActiveGame::new(session, 0, map))
}

fn start_join_session(
    local_port: u16,
    host_addr: SocketAddr,
    map: Map,
) -> Result<ActiveGame, String> {
    let socket = UdpNonBlockingSocket::bind_to_port(local_port)
        .map_err(|e| format!("GGRS Bind Error: {:?}", e))?;

    let session = SessionBuilder::<GgrsConfig>::new()
        .with_num_players(2)
        .unwrap()
        .with_fps(FPS as usize)
        .unwrap()
        .with_input_delay(1)
        .add_player(PlayerType::Remote(host_addr), 0)
        .unwrap()
        .add_player(PlayerType::Local, 1)
        .unwrap()
        .start_p2p_session(socket)
        .unwrap();

    Ok(ActiveGame::new(session, 1, map))
}
