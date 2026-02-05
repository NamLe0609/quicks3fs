pub mod s3_benchmarks;
pub mod s3_client;
use s3_client::{AwsCredential, S3Request, make_client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = "shee-vm1:9000";
    let endpoint = "test/secret-msg";
    let uri = format!("https://shee-vm1:8443/{}", endpoint)
        .parse::<http::Uri>()
        .unwrap();
    let http_version = http::Version::HTTP_3;

    let access_key = "root";
    let secret_key = "beak-lamp-blind";
    let region = "us-east-1";

    let aws_cred = AwsCredential::new(access_key, secret_key, region);
    let reqwest_req = S3Request::new(http::Method::GET, uri, http_version)
        .header(
            http::header::HOST,
            http::HeaderValue::from_str(host).unwrap(),
        )
        .build_and_sign(aws_cred)
        .unwrap();

    let client = make_client(http_version)?;
    let res = client.execute(reqwest_req).await?;

    println!("Status: {}", res.status());
    println!("Body: {}", res.text().await?);

    Ok(())
}
