use std::{
    fmt,
    net::{TcpStream, ToSocketAddrs, UdpSocket},
    time::Duration,
};

use tracing::trace;

/// Try to open a websocket connection to the target address
fn is_server_running<A: ToSocketAddrs>(address: A, timeout: Duration) -> bool {
    TcpStream::connect_timeout(&address.to_socket_addrs().unwrap().next().unwrap(), timeout).is_ok()
}

/// Build the magic packet to wake the server
fn create_magic_packet(mac_addr: &MacAddr6) -> anyhow::Result<Vec<u8>> {
    // Define the header
    trace!("Creating magic packet");
    let mut packet = vec![0xff; 6];

    // Loop mac_addr bytes 16 times and append to header
    for _ in 0..16 {
        packet.extend_from_slice(mac_addr.as_bytes());
    }

    Ok(packet)
}

pub async fn send_wol_packet(mac_addr: &MacAddr6) -> anyhow::Result<()> {
    let magic_packet = create_magic_packet(mac_addr)?;

    trace!("Binding socket");
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    trace!("Setting to broadcast");
    socket.set_broadcast(true)?;
    let target = "192.168.1.255:9";
    trace!("Sending wol packet");
    socket.send_to(&magic_packet, target)?;
    Ok(())
}

#[derive(Debug, Clone, Hash, Ord, PartialOrd, PartialEq, Eq)]
pub enum MacParseError {
    /// Invalid length of string
    InvalidLength(usize),
    /// Invalid character in string
    InvalidChar(String),
}

impl fmt::Display for MacParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MacParseError::InvalidLength(len) => {
                f.write_fmt(format_args!("Invalid length: {}", len))
            }
            MacParseError::InvalidChar(s) => f.write_fmt(format_args!("Invalid chars: {}", s)),
        }
    }
}

impl std::error::Error for MacParseError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddr6([u8; 6]);

impl TryFrom<&str> for MacAddr6 {
    type Error = MacParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let addr = parse_mac6_addr(value)?;

        Ok(Self(addr))
    }
}

impl std::fmt::Display for MacAddr6 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5],
        ))
    }
}

impl MacAddr6 {
    pub fn as_bytes(&self) -> &[u8; 6] {
        &self.0
    }
}

// TODO: Implement
pub struct MacAddr8 {}

pub fn parse_mac6_addr(mac: &str) -> Result<[u8; 6], MacParseError> {
    let mac_str = mac.replace(":", "");

    if mac_str.len() != 12 {
        return Err(MacParseError::InvalidLength(mac_str.len()));
    }

    let mut mac_bytes_6 = [0u8; 6];

    for i in 0..6 {
        let pair = &mac_str[i * 2..i * 2 + 2];
        let Ok(val) = u8::from_str_radix(pair, 16) else {
            return Err(MacParseError::InvalidChar(pair.to_string()));
        };
        mac_bytes_6[i] = val;
    }

    Ok(mac_bytes_6)
}
