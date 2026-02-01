use core_affinity;
use ctrlc;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

fn main() {
    sender().unwrap()
}

fn sender() -> std::io::Result<()> {
    // Pin receiver to core 0
    let core_ids = core_affinity::get_core_ids().unwrap();
    if let Some(&core_id) = core_ids.first() {
        core_affinity::set_for_current(core_id);
    }

    // Let CTRL+C quit but also output results
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // Socket setup
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    let buffer_size = 8 * 1024 * 1024;
    socket.set_send_buffer_size(buffer_size)?;
    socket.set_nonblocking(true)?;

    let receiver_address: SocketAddr = "192.168.8.227:12345".parse().unwrap();

    println!("Press CTRL+C to stop sending...");

    let mut buf = [0u8; 1500];
    let start = Instant::now();
    let mut packet_sent = 0;
    let mut bytes_sent = 0;

    while running.load(Ordering::SeqCst) {
        match socket.send_to(&mut buf, &receiver_address.into()) {
            Ok(number_of_bytes) => {
                packet_sent += 1;
                bytes_sent += number_of_bytes;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e),
        }
    }

    let duration = start.elapsed();
    let seconds = duration.as_secs_f64();

    println!("Test ran for {:.2} seconds", seconds);
    println!("Packets sent: {}", packet_sent);
    println!("Bytes sent: {}", bytes_sent);
    println!(
        "Throughput: {:.2} MB/s",
        bytes_sent as f64 / 1_048_576.0 / seconds
    );
    println!("Rate: {:.0} packets/sec", packet_sent as f64 / seconds);

    Ok(())
}
