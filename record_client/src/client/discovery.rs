use std::net::{UdpSocket, UdpBuilder, Ipv4Addr};
use std::io;

fn discover_server(discovery_ip: Ipv4Addr, port: u16) -> Result<SocketAddr, io::Error> {
    let udp: UdpSocket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], port)))?;
    udp.join_multicast_v4(&discovery_ip, &Ipv4Addr::new(0, 0, 0, 0))?;

    let mut buffer = [0u8; 32];

    loop {
        let (size, addr) = udp.recv_from(&mut buffer)?;
        // header verification - 4 bytes (28)
        if buffer.starts_with(&[0xE, 0xE, 0xA, 0xB]) {
            // session code - next 4 bytes (24)
            let session_code = buffer[4..8];
            // port - next 2 bytes (22)
            let byte_one = buffer[9];
            let byte_two = buffer[10] as u16;
            let port = ;

            return Ok(SocketAddr::from((addr.ip(), port))?)
        }
    }
}