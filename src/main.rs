use aws_credential_types::Credentials;
use aws_sigv4::{
    http_request::{SignableBody, SignableRequest, SigningSettings, sign},
    sign::v4,
};
use http::Version;
use reqwest::Client;
use std::time::SystemTime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = "shee-vm1:9000";
    let endpoint = "test/secret-msg";
    // let endpoint = "minio/health/live";
    let url = format!("https://shee-vm1:8443/{}", endpoint);
    let access_key = "root";
    let secret_key = "beak-lamp-blind";
    let region = "us-east-1";
    let service = "s3";

    // Create the HTTP request
    let mut request = http::Request::builder()
        .method("GET")
        .header("host", host)
        .version(Version::HTTP_3)
        .uri(&url)
        .body("")?;

    // let mut request = http::Request::builder()
    //     .method("GET")
    //     .header("host", host)
    //     .version(Version::HTTP_2)
    //     .uri(&url)
    //     .body("")?;

    let creds = Credentials::new(access_key, secret_key, None, None, service);
    let identity = creds.into();

    // Set up signing parameters
    let signing_settings = SigningSettings::default();
    let signing_params = v4::SigningParams::builder()
        .identity(&identity)
        .region(region)
        .name(service)
        .time(SystemTime::now())
        .settings(signing_settings)
        .build()?
        .into();

    let signable_request = SignableRequest::new(
        request.method().as_str(),
        request.uri().to_string(),
        request
            .headers()
            .iter()
            .map(|(k, v)| (k.as_str(), std::str::from_utf8(v.as_bytes()).unwrap())),
        SignableBody::Bytes(request.body().as_bytes()),
    )?;

    // Sign the request
    let (signing_instructions, _signature) = sign(signable_request, &signing_params)?.into_parts();
    signing_instructions.apply_to_request_http1x(&mut request);

    request.headers_mut().remove("host");
    let reqwest_req: reqwest::Request = request.try_into()?;
    let client = Client::builder()
        .tls_danger_accept_invalid_certs(true)
        .http3_prior_knowledge()
        .http3_send_grease(false)
        .build()?;
    // let client = Client::builder()
    //     .tls_danger_accept_invalid_certs(true)
    //     .http2_prior_knowledge()
    //     .build()?;
    println!("Request version: {:?}", reqwest_req.version());
    let res = client.execute(reqwest_req).await?;

    println!("Status: {}", res.status());
    println!("Body: {}", res.text().await?);

    Ok(())
}
