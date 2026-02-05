pub mod s3_benchmarks;
pub mod s3_client;

use crate::s3_benchmarks::{
    NewConnectionThroughput, ReuseConnectionThroughput, TimeToFirstByteLatency, TimeToHeaderLatency,
};
use crate::s3_client::AwsCredential;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let access_key = "root";
    let secret_key = "beak-lamp-blind";
    let region = "us-east-1";
    let aws_cred = AwsCredential::new(access_key, secret_key, region);

    let new_connection_throughput_http2 =
        NewConnectionThroughput::new(http::Version::HTTP_2, aws_cred.clone());
    let new_connection_throughput_http3 =
        NewConnectionThroughput::new(http::Version::HTTP_3, aws_cred.clone());

    let reuse_connection_throughput_http2 =
        ReuseConnectionThroughput::new(http::Version::HTTP_2, aws_cred.clone());
    let reuse_connection_throughput_http3 =
        ReuseConnectionThroughput::new(http::Version::HTTP_3, aws_cred.clone());

    let time_to_header_latency_http2 =
        TimeToHeaderLatency::new(http::Version::HTTP_2, aws_cred.clone());
    let time_to_header_latency_http3 =
        TimeToHeaderLatency::new(http::Version::HTTP_3, aws_cred.clone());

    let time_to_first_byte_latency_http2 =
        TimeToFirstByteLatency::new(http::Version::HTTP_2, aws_cred.clone());
    let time_to_first_byte_latency_http3 =
        TimeToFirstByteLatency::new(http::Version::HTTP_2, aws_cred.clone());

    // println!("New connection per request HTTP2: ");
    // new_connection_throughput_http2.test_all_files().await?;
    // println!("New connection per request HTTP3: ");
    // new_connection_throughput_http3.test_all_files().await?;
    // println!("Reused connection per request HTTP2: ");
    // reuse_connection_throughput_http2.test_all_files().await?;
    // println!("Reused connection per request HTTP3: ");
    // reuse_connection_throughput_http3.test_all_files().await?;
    // println!("Time to header latency HTTP2: ");
    // time_to_header_latency_http2.test_all_files().await?;
    // println!("Time to header latency HTTP3: ");
    // time_to_header_latency_http3.test_all_files().await?;
    // println!("Time to first byte latency HTTP2: ");
    // time_to_first_byte_latency_http2.test_all_files().await?;
    // println!("Time to first byte latency HTTP3: ");
    // time_to_first_byte_latency_http3.test_all_files().await?;
    Ok(())
}
