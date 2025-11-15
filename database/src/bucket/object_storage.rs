#![allow(unused, clippy::too_many_arguments)]

use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::{Digest, Sha256};

pub struct ObjectStorage {
    pub client: Client,
    pub access_key: String,
    pub secret_key: String,
    pub endpoint: String,
    pub name: String,
    pub region: String,
    pub public_url: String,
}

impl Default for ObjectStorage {
    fn default() -> Self {
        Self {
            client: Client::new(),
            access_key: std::env::var("BUCKET_ACCESS_KEY").unwrap(),
            secret_key: std::env::var("BUCKET_SECRET_KEY").unwrap(),
            endpoint: std::env::var("BUCKET_ENDPOINT").unwrap(),
            region: std::env::var("BUCKET_REGION").unwrap(),
            name: std::env::var("BUCKET_NAME").unwrap(),
            public_url: std::env::var("BUCKET_PUBLIC_URL").unwrap(),
        }
    }
}

// impl ObjectStorage {
//     fn generate_signature(
//         &self,
//         method: &str,
//         path: &str,
//         date: &str,
//         content_type: &str,
//         content_md5: &str,
//     ) -> String {
//         let string_to_sign = format!(
//             "{}\n{}\n{}\n{}\n{}",
//             method, content_md5, content_type, date, path
//         );

//         let mut mac = Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes())
//             .expect("HMAC can take key of any size");
//         mac.update(string_to_sign.as_bytes());
//         let result = mac.finalize();
//         base64::Engine::encode(
//             &base64::engine::general_purpose::STANDARD,
//             result.into_bytes(),
//         )
//     }

//     fn generate_aws_v4_signature(
//         &self,
//         method: &str,
//         path: &str,
//         query: &str,
//         headers: &[(&str, &str)],
//         payload_hash: &str,
//         date: &str,
//         datetime: &str,
//     ) -> (String, String) {
//         // Canonical headers
//         let mut canonical_headers = headers
//             .iter()
//             .map(|(k, v)| format!("{}:{}", k.to_lowercase(), v.trim()))
//             .collect::<Vec<_>>();
//         canonical_headers.sort();
//         let canonical_headers_str = canonical_headers.join("\n") + "\n";

//         let signed_headers = headers
//             .iter()
//             .map(|(k, _)| k.to_lowercase())
//             .collect::<Vec<_>>()
//             .join(";");

//         // Canonical request
//         let canonical_request = format!(
//             "{}\n{}\n{}\n{}\n{}\n{}",
//             method, path, query, canonical_headers_str, signed_headers, payload_hash
//         );

//         let mut hasher = Sha256::new();
//         hasher.update(canonical_request.as_bytes());
//         let canonical_request_hash = const_hex::encode(hasher.finalize());

//         // String to sign
//         let credential_scope = format!("{}/{}/s3/aws4_request", date, self.region);
//         let string_to_sign = format!(
//             "AWS4-HMAC-SHA256\n{}\n{}\n{}",
//             datetime, credential_scope, canonical_request_hash
//         );

//         // Signing key
//         let k_date = hmac_sha256(
//             format!("AWS4{}", self.secret_key).as_bytes(),
//             date.as_bytes(),
//         );
//         let k_region = hmac_sha256(&k_date, self.region.as_bytes());
//         let k_service = hmac_sha256(&k_region, b"s3");
//         let k_signing = hmac_sha256(&k_service, b"aws4_request");

//         let signature = const_hex::encode(hmac_sha256(&k_signing, string_to_sign.as_bytes()));

//         let authorization = format!(
//             "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
//             self.access_key, credential_scope, signed_headers, signature
//         );

//         (authorization, signed_headers)
//     }
// }

// fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
//     let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
//     mac.update(data);
//     mac.finalize().into_bytes().to_vec()
// }
