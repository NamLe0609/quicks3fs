use ctrlc;
use socket2::{Domain, Protocol, Socket, Type};
use std::io;
use std::mem::MaybeUninit;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

fn main() {}

fn server() -> std::io::Result<()> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    let buffer_size = 8 * 1024 * 1024;
    socket.set_recv_buffer_size(buffer_size)?;
    let mut buf = [MaybeUninit::<u8>::uninit(); 1500];

    let address: SocketAddr = "192.168.8.227:12345".parse().unwrap();

    println!("Press Enter when ready to start the test...");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    socket.bind(&address.into())?;
    let timer = Instant::now();
    let test_seconds = 10;
    let test_duration = Duration::from_secs(test_seconds);

    println!("Starting test...");

    let mut packet_received = 0;
    let mut bytes_received = 0;
    while timer.elapsed() < test_duration {
        let number_of_bytes = socket.recv(&mut buf)?;
        packet_received += 1;
        bytes_received += number_of_bytes;
    }

    let duration = timer.elapsed();
    let seconds = duration.as_secs_f64();

    println!("Test ran for {:.2} seconds", seconds);
    println!("Packets received: {}", packet_received);
    println!("Bytes received: {}", bytes_received);
    println!(
        "Throughput: {:.2} MB/s",
        bytes_received as f64 / 1_048_576.0 / seconds
    );
    println!("Rate: {:.0} packets/sec", packet_received as f64 / seconds);
    Ok(())
}

fn client() -> std::io::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    let buffer_size = 8 * 1024 * 1024;
    socket.set_send_buffer_size(buffer_size)?;
    let mut buf = [0u8; 1500];

    let receiver_address: SocketAddr = "192.168.8.227:12345".parse().unwrap();

    println!("Press CTRL+C to stop sending...");

    let start = Instant::now();
    let mut packet_sent = 0;
    let mut bytes_sent = 0;

    while running.load(Ordering::SeqCst) {
        let number_of_bytes = socket.send_to(&mut buf, &receiver_address.into())?;
        packet_sent += 1;
        bytes_sent += number_of_bytes;
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
