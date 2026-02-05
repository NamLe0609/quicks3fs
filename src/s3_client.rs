use aws_credential_types::Credentials;
use aws_sigv4::{
    http_request::{SignableBody, SignableRequest, SigningSettings, sign},
    sign::v4,
};
use http::{HeaderMap, HeaderName, HeaderValue, Method, Request, Uri, Version};
use std::time::SystemTime;

pub struct AwsCredential {
    access_key: String,
    region: String,
    secret_key: String,
}

impl AwsCredential {
    pub fn new(
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
        region: impl Into<String>,
    ) -> Self {
        Self {
            access_key: access_key.into(),
            secret_key: secret_key.into(),
            region: region.into(),
        }
    }
}

pub struct S3Request {
    body: Vec<u8>,
    headers: HeaderMap,
    method: Method,
    uri: Uri,
    version: Version,
}

impl S3Request {
    pub fn new(
        method: impl Into<Method>,
        uri: impl Into<Uri>,
        version: impl Into<Version>,
    ) -> Self {
        let method = method.into();
        let uri = uri.into();
        let version = version.into();
        let headers = HeaderMap::new();

        Self {
            method,
            uri,
            version,
            headers,
            body: Vec::new(),
        }
    }

    pub fn header(mut self, key: HeaderName, value: HeaderValue) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    pub fn build_and_sign(
        self,
        credentials: AwsCredential,
    ) -> Result<reqwest::Request, Box<dyn std::error::Error>> {
        let mut request = Request::builder()
            .method(self.method)
            .version(self.version)
            .uri(&self.uri)
            .body(self.body)?;

        *request.headers_mut() = self.headers;

        let creds = Credentials::new(
            credentials.access_key,
            credentials.secret_key,
            None,
            None,
            "s3",
        );
        let identity = creds.into();

        // Set up signing parameters
        let signing_settings = SigningSettings::default();
        let signing_params = v4::SigningParams::builder()
            .identity(&identity)
            .region(credentials.region.as_str())
            .name("s3")
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
            SignableBody::Bytes(request.body()),
        )?;

        // Sign the request
        let (signing_instructions, _signature) =
            sign(signable_request, &signing_params)?.into_parts();
        signing_instructions.apply_to_request_http1x(&mut request);

        request.headers_mut().remove("host");
        let reqwest_req: reqwest::Request = request.try_into()?;
        Ok(reqwest_req)
    }
}

pub fn make_client(version: Version) -> Result<reqwest::Client, reqwest::Error> {
    let client = match version {
        Version::HTTP_2 => reqwest::Client::builder()
            .tls_danger_accept_invalid_certs(true)
            .http2_prior_knowledge()
            .build()?,
        Version::HTTP_3 => reqwest::Client::builder()
            .tls_danger_accept_invalid_certs(true)
            .http3_prior_knowledge()
            .http3_send_grease(false)
            .build()?,
        _ => {
            panic!()
        }
    };
    Ok(client)
}
