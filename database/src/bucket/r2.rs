#![allow(clippy::too_many_arguments)]

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use util::AppError;

pub struct CloudflareR2 {
    pub client: reqwest::Client,
    pub access_key: String,
    pub secret_key: String,
    pub endpoint: String,
    pub name: String,
    pub region: String,
    pub public_url: String,
}

impl Default for CloudflareR2 {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            access_key: std::env::var("BUCKET_ACCESS_KEY").unwrap(),
            secret_key: std::env::var("BUCKET_SECRET_KEY").unwrap(),
            endpoint: std::env::var("BUCKET_ENDPOINT").unwrap(),
            region: std::env::var("BUCKET_REGION").unwrap(),
            name: std::env::var("BUCKET_NAME").unwrap(),
            public_url: std::env::var("BUCKET_PUBLIC_URL").unwrap(),
        }
    }
}

impl CloudflareR2 {
    fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    fn generate_aws_v4_signature(
        &self,
        method: &str,
        path: &str,
        query: &str,
        headers: &[(&str, &str)],
        payload_hash: &str,
        date: &str,
        datetime: &str,
    ) -> (String, String) {
        // Canonical headers
        let mut canonical_headers = headers
            .iter()
            .map(|(k, v)| format!("{}:{}", k.to_lowercase(), v.trim()))
            .collect::<Vec<_>>();
        canonical_headers.sort();
        let canonical_headers_str = canonical_headers.join("\n") + "\n";

        let signed_headers =
            headers.iter().map(|(k, _)| k.to_lowercase()).collect::<Vec<_>>().join(";");

        // Canonical request
        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            method, path, query, canonical_headers_str, signed_headers, payload_hash
        );

        let mut hasher = Sha256::new();
        hasher.update(canonical_request.as_bytes());
        let canonical_request_hash = const_hex::encode(hasher.finalize());

        // String to sign
        let credential_scope = format!("{}/{}/s3/aws4_request", date, self.region);
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{}\n{}\n{}",
            datetime, credential_scope, canonical_request_hash
        );

        // Signing key
        let k_date =
            Self::hmac_sha256(format!("AWS4{}", self.secret_key).as_bytes(), date.as_bytes());
        let k_region = Self::hmac_sha256(&k_date, self.region.as_bytes());
        let k_service = Self::hmac_sha256(&k_region, b"s3");
        let k_signing = Self::hmac_sha256(&k_service, b"aws4_request");

        let signature = const_hex::encode(Self::hmac_sha256(&k_signing, string_to_sign.as_bytes()));

        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
            self.access_key, credential_scope, signed_headers, signature
        );

        (authorization, signed_headers)
    }

    /// Generates signed headers for AWS S3 API requests using AWS Signature Version 4.
    /// Returns a vector of header tuples (header_name, header_value).
    ///
    /// # Arguments
    /// * `method` - HTTP method (GET, PUT, DELETE, etc.)
    /// * `path` - URL path (e.g., "/filename.jpg")
    /// * `query` - Query string (empty for most requests)
    /// * `content_type` - Optional content type
    /// * `payload_hash` - SHA256 hash of the request body (or empty string hash for DELETE)
    pub fn get_signed_headers(
        &self,
        method: &str,
        path: &str,
        query: &str,
        content_type: Option<&str>,
        payload_hash: &str,
    ) -> Vec<(&'static str, String)> {
        let now = OffsetDateTime::now_utc();

        // Format: YYYYMMDD
        let date = format!("{:04}{:02}{:02}", now.year(), now.month() as u8, now.day());

        // Format: YYYYMMDDTHHMMSSZ
        let datetime = format!(
            "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
            now.year(),
            now.month() as u8,
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        );

        // For R2, the host should be the endpoint without https:// or http://
        let host = self.endpoint.trim_start_matches("https://").trim_start_matches("http://");

        let mut headers: Vec<(&str, String)> = vec![
            ("host", host.to_string()),
            ("x-amz-content-sha256", payload_hash.to_string()),
            ("x-amz-date", datetime.clone()),
        ];

        if let Some(ct) = content_type {
            headers.push(("content-type", ct.to_string()));
        }

        let header_refs: Vec<(&str, &str)> = headers.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let (authorization, _signed_headers_str) = self.generate_aws_v4_signature(
            method,
            path,
            query,
            &header_refs,
            payload_hash,
            &date,
            &datetime,
        );

        headers.push(("authorization", authorization));

        headers
    }

    /// Uploads a file to Cloudflare R2 (S3-compatible storage)
    pub async fn upload_file(
        &self,
        data: axum::body::Bytes,
        filename: &str,
        content_type: &str,
    ) -> Result<String, AppError> {
        if data.is_empty() {
            tracing::error!("Empty file data for: {}", filename);
            return Err(AppError::ServerError);
        }

        // Calculate SHA256 hash of payload
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let payload_hash = const_hex::encode(hasher.finalize());

        let path = format!("/{}/{}", self.name, filename);
        let headers = self.get_signed_headers("PUT", &path, "", Some(content_type), &payload_hash);

        let url = format!("{}/{}/{}", self.endpoint, self.name, filename);

        let mut request = self
            .client
            .put(&url)
            .header("Content-Type", content_type)
            .header("Content-Length", data.len())
            .body(data.to_vec());

        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request.send().await.map_err(|e| {
            tracing::error!("Upload failed for {}: {:#?}", filename, e);
            AppError::ServerError
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Upload error for {} ({}): {}", filename, status, error_text);
            return Err(AppError::ServerError);
        }

        Ok(format!("{}/{}", self.public_url, filename))
    }

    /// Deletes a file from Cloudflare R2
    pub async fn delete_file(&self, filename: &str) -> Result<(), AppError> {
        let path = format!("/{}/{}", self.name, filename);
        let headers = self.get_signed_headers(
            "DELETE",
            &path,
            "",
            None,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", // Empty string SHA256
        );

        let url = format!("{}/{}/{}", self.endpoint, self.name, filename);

        let mut request = self.client.delete(&url);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request.send().await.map_err(|e| {
            tracing::error!("Delete failed for {}: {:#?}", filename, e);
            AppError::ServerError
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Delete error for {} ({}): {}", filename, status, error_text);
            return Err(AppError::ServerError);
        }

        Ok(())
    }
}
