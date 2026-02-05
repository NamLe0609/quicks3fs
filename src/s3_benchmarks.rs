use crate::s3_client::{AwsCredential, S3Request, make_client};
use std::time::{Duration, Instant};

// pub struct BenchmarkResults {
//     pub protocol: http::Version,

//     // Throughput (MB/s)
//     pub small_sequential: f64,
//     pub medium_sequential: f64,
//     pub large_sequential: f64,

//     pub small_concurrent: f64,
//     pub medium_concurrent: f64,
//     pub large_concurrent: f64,
//     pub mixed_concurrent: f64,

//     // Connection reuse
//     pub small_reused_gain: f64,
//     pub medium_reused_gain: f64,
//     pub large_reused_gain: f64,

//     // Connection init latency
//     pub avg_latency_ms: f64,

//     // Time-to-first-byte latency
//     pub avg_ttfb_ms: f64,
// }

// impl BenchmarkResults {
//     pub fn print_table(&self) {
//         println!("\n=== {:?} Performance ===", self.protocol);
//         println!(
//             "{:<20} {:>10} {:>10} {:>10}",
//             "Test", "Small", "Medium", "Large"
//         );
//         println!("{:-<60}", "");
//         println!(
//             "{:<20} {:>10.1} {:>10.1} {:>10.1}",
//             "Sequential MB/s", self.small_sequential, self.medium_sequential, self.large_sequential
//         );
//         println!(
//             "{:<20} {:>10.1} {:>10.1} {:>10.1}",
//             "Concurrent MB/s", self.small_concurrent, self.medium_concurrent, self.large_concurrent
//         );
//         println!(
//             "{:<20} {:>10.1}% {:>10.1}% {:>10.1}%",
//             "Reuse Gain", self.small_reused_gain, self.medium_reused_gain, self.large_reused_gain
//         );
//         println!("\nMixed Workload: {:.1} MB/s", self.mixed_concurrent);
//         println!("Avg Latency: {:.1}ms", self.avg_latency_ms);
//         println!("Avg TTFB: {:.1}ms", self.avg_ttfb_ms);
//     }
// }

// Small files (< 1MB)
const SMALL_SIZES: [u64; 8] = [4, 8, 16, 32, 64, 128, 256, 512];
// Medium files (1MB - 512MB)
const MEDIUM_SIZES: [u64; 10] = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512];
// Large files (1GB+)
const LARGE_SIZES: [u64; 4] = [1, 2, 4, 8];

enum FileCategory {
    Small,
    Medium,
    Large,
}

impl FileCategory {
    fn array(&self) -> &'static [u64] {
        match self {
            FileCategory::Small => &SMALL_SIZES,
            FileCategory::Medium => &MEDIUM_SIZES,
            FileCategory::Large => &LARGE_SIZES,
        }
    }

    fn unit(&self) -> &'static str {
        match self {
            FileCategory::Small => "KiB",
            FileCategory::Medium => "MiB",
            FileCategory::Large => "GiB",
        }
    }

    fn multiplier(&self) -> u64 {
        match self {
            FileCategory::Small => 1024,
            FileCategory::Medium => 1024 * 1024,
            FileCategory::Large => 1024 * 1024 * 1024,
        }
    }

    fn total_bytes(&self) -> u64 {
        self.array()
            .iter()
            .map(|&size| size * self.multiplier())
            .sum()
    }

    fn throughput_mib_per_sec(&self, duration: Duration) -> f64 {
        self.total_bytes() as f64 / duration.as_secs_f64() / (1024.0 * 1024.0)
    }
}

pub struct NewConnectionThroughput {
    http_version: http::Version,
    aws_creds: AwsCredential,
}

impl NewConnectionThroughput {
    pub fn new(version: http::Version, aws_cred: AwsCredential) -> Self {
        Self {
            http_version: version,
            aws_creds: aws_cred,
        }
    }

    pub async fn test_all_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        let host = "shee-vm1:9000";
        let file_category = [
            FileCategory::Small,
            FileCategory::Medium,
            FileCategory::Large,
        ];
        for category in file_category {
            let mut category_time = Duration::ZERO;
            for size in category.array() {
                let uri = format!(
                    "https://shee-vm1:8443/test/file-{}{}.dat",
                    size,
                    category.unit()
                )
                .parse::<http::Uri>()?;
                let req = S3Request::new(http::Method::GET, uri, self.http_version)
                    .header(http::header::HOST, http::HeaderValue::from_str(host)?)
                    .build_and_sign(&self.aws_creds)?;

                let request_start = Instant::now();
                let client = make_client(self.http_version)?;
                let res = client.execute(req).await?;
                res.text().await?;
                category_time += request_start.elapsed();
            }
            match category {
                FileCategory::Small => {
                    println!(
                        "Small file throughput (<1MiB): {:.2} MiB/s",
                        category.throughput_mib_per_sec(category_time)
                    );
                }
                FileCategory::Medium => {
                    println!(
                        "Medium file throughput (1MiB-1GiB): {:.2} MiB/s",
                        category.throughput_mib_per_sec(category_time)
                    );
                }
                FileCategory::Large => {
                    println!(
                        "Large file throughput (1GiB-4GiB): {:.2} MiB/s",
                        category.throughput_mib_per_sec(category_time)
                    );
                }
            }
        }
        Ok(())
    }
}

pub struct ReuseConnectionThroughput {
    http_version: http::Version,
    aws_creds: AwsCredential,
}

impl ReuseConnectionThroughput {
    pub fn new(version: http::Version, aws_cred: AwsCredential) -> Self {
        Self {
            http_version: version,
            aws_creds: aws_cred,
        }
    }

    pub async fn test_all_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        let host = "shee-vm1:9000";
        let file_category = [
            FileCategory::Small,
            FileCategory::Medium,
            FileCategory::Large,
        ];
        let client = make_client(self.http_version)?;
        for category in file_category {
            let mut category_time = Duration::ZERO;
            for size in category.array() {
                let uri = format!(
                    "https://shee-vm1:8443/test/file-{}{}.dat",
                    size,
                    category.unit()
                )
                .parse::<http::Uri>()?;
                let req = S3Request::new(http::Method::GET, uri, self.http_version)
                    .header(http::header::HOST, http::HeaderValue::from_str(host)?)
                    .build_and_sign(&self.aws_creds)?;

                let request_start = Instant::now();
                let res = client.execute(req).await?;
                res.text().await?;
                category_time += request_start.elapsed();
            }
            match category {
                FileCategory::Small => {
                    println!(
                        "Small file throughput (<1MiB): {:.2} MiB/s",
                        category.throughput_mib_per_sec(category_time)
                    );
                }
                FileCategory::Medium => {
                    println!(
                        "Medium file throughput (1MiB-1GiB): {:.2} MiB/s",
                        category.throughput_mib_per_sec(category_time)
                    );
                }
                FileCategory::Large => {
                    println!(
                        "Large file throughput (1GiB-4GiB): {:.2} MiB/s",
                        category.throughput_mib_per_sec(category_time)
                    );
                }
            }
        }
        Ok(())
    }
}

pub struct TimeToHeaderLatency {
    http_version: http::Version,
    aws_creds: AwsCredential,
}

impl TimeToHeaderLatency {
    pub fn new(version: http::Version, aws_cred: AwsCredential) -> Self {
        Self {
            http_version: version,
            aws_creds: aws_cred,
        }
    }

    pub async fn test_all_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        let host = "shee-vm1:9000";
        let category = FileCategory::Small;
        let mut category_time = Duration::ZERO;
        let no_iterations = 100;
        for _ in 0..no_iterations {
            for size in category.array() {
                let uri = format!(
                    "https://shee-vm1:8443/test/file-{}{}.dat",
                    size,
                    category.unit()
                )
                .parse::<http::Uri>()?;
                let req = S3Request::new(http::Method::GET, uri, self.http_version)
                    .header(http::header::HOST, http::HeaderValue::from_str(host)?)
                    .build_and_sign(&self.aws_creds)?;
                let client = make_client(self.http_version)?;

                let request_start = Instant::now();
                client.execute(req).await?;
                category_time += request_start.elapsed();
            }
        }
        println!(
            "Connection initialization latency: {:.2} ms",
            category_time.as_micros() / (category.array().len() * no_iterations) as u128
        );

        Ok(())
    }
}

pub struct TimeToFirstByteLatency {
    http_version: http::Version,
    aws_creds: AwsCredential,
}

impl TimeToFirstByteLatency {
    pub fn new(version: http::Version, aws_cred: AwsCredential) -> Self {
        Self {
            http_version: version,
            aws_creds: aws_cred,
        }
    }

    pub async fn test_all_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        let host = "shee-vm1:9000";
        let category = FileCategory::Small;
        let mut category_time = Duration::ZERO;
        let no_iterations = 100;
        for _ in 0..no_iterations {
            for size in category.array() {
                let uri = format!(
                    "https://shee-vm1:8443/test/file-{}{}.dat",
                    size,
                    category.unit()
                )
                .parse::<http::Uri>()?;
                let req = S3Request::new(http::Method::GET, uri, self.http_version)
                    .header(http::header::HOST, http::HeaderValue::from_str(host)?)
                    .build_and_sign(&self.aws_creds)?;
                let client = make_client(self.http_version)?;

                let request_start = Instant::now();
                let mut res = client.execute(req).await?;
                res.chunk().await?;
                category_time += request_start.elapsed();
            }
        }
        println!(
            "Connection initialization latency: {:.2} ms",
            category_time.as_micros() / (category.array().len() * no_iterations) as u128
        );

        Ok(())
    }
}
