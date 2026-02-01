use socket2::{Domain, Protocol, Socket, Type};
use std::io;
use std::mem::MaybeUninit;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

fn main() {
    receiver().unwrap()
}

fn receiver() -> std::io::Result<()> {
    // Pin receiver to core 0
    let core_ids = core_affinity::get_core_ids().unwrap();
    if let Some(&core_id) = core_ids.first() {
        core_affinity::set_for_current(core_id);
    }

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    let buffer_size = 8 * 1024 * 1024;
    socket.set_recv_buffer_size(buffer_size)?;
    socket.set_nonblocking(true)?;
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
        match socket.recv(&mut buf) {
            Ok(number_of_bytes) => {
                packet_received += 1;
                bytes_received += number_of_bytes;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(e) => {
                return Err(e);
            }
        }
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
