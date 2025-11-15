// use common::AppError;
// use sha2::Sha256;
// use std::sync::Arc;

// impl crate::Db {
//     pub async fn upload_image(
//         self: &Arc<Self>,
//         filename: &str,
//         data: axum::body::Bytes,
//         content_type: &str,
//     ) -> Result<String, AppError> {
//         let url = format!(
//             "{}/{}/{}",
//             self.bucket.endpoint.trim_end_matches('/'),
//             self.bucket.name,
//             filename
//         );

//         let now = Utc::now();
//         let date = now.format("%Y%m%d").to_string();
//         let datetime = now.format("%Y%m%dT%H%M%SZ").to_string();

//         // Calculate payload hash
//         let mut hasher = Sha256::new();
//         hasher.update(&data);
//         let payload_hash = hex::encode(hasher.finalize());

//         let path = format!("/{}/{}", self.bucket.name, filename);
//         let headers = vec![
//             (
//                 "host",
//                 self.bucket
//                     .endpoint
//                     .trim_start_matches("https://")
//                     .trim_start_matches("http://"),
//             ),
//             ("x-amz-content-sha256", payload_hash.as_str()),
//             ("x-amz-date", datetime.as_str()),
//             ("content-type", content_type),
//         ];

//         let (authorization, _) = self.bucket.generate_aws_v4_signature(
//             "PUT",
//             &path,
//             "",
//             &headers,
//             &payload_hash,
//             &date,
//             &datetime,
//         );

//         let response = self
//             .bucket
//             .client
//             .put(&url)
//             .header("Authorization", authorization)
//             .header("x-amz-content-sha256", payload_hash)
//             .header("x-amz-date", datetime)
//             .header("Content-Type", content_type)
//             .header("Content-Length", data.len())
//             .body(data.to_vec())
//             .send()
//             .await
//             .map_err(|e| {
//                 tracing::error!("Upload error: {e:#?}");
//                 AppError::ServerError
//             })?;

//         if !response.status().is_success() {
//             let status = response.status();
//             let body = response.text().await.unwrap_or_default();
//             tracing::error!("S3 upload failed: {} - {}", status, body);
//             return Err(AppError::ServerError);
//         }

//         Ok(format!("{}/{}", self.bucket.public_url, filename))
//     }

//     pub async fn delete_image(self: &Arc<Self>, filename: &str) -> Result<(), AppError> {
//         let url = format!(
//             "{}/{}/{}",
//             self.bucket.endpoint.trim_end_matches('/'),
//             self.bucket.name,
//             filename
//         );

//         let now = Utc::now();
//         let date = now.format("%Y%m%d").to_string();
//         let datetime = now.format("%Y%m%dT%H%M%SZ").to_string();

//         let payload_hash = const_hex::encode(Sha256::digest(b""));

//         let path = format!("/{}/{}", self.bucket.name, filename);
//         let headers = vec![
//             (
//                 "host",
//                 self.bucket
//                     .endpoint
//                     .trim_start_matches("https://")
//                     .trim_start_matches("http://"),
//             ),
//             ("x-amz-content-sha256", payload_hash.as_str()),
//             ("x-amz-date", datetime.as_str()),
//         ];

//         let (authorization, _) = self.bucket.generate_aws_v4_signature(
//             "DELETE",
//             &path,
//             "",
//             &headers,
//             &payload_hash,
//             &date,
//             &datetime,
//         );

//         let response = self
//             .bucket
//             .client
//             .delete(&url)
//             .header("Authorization", authorization)
//             .header("x-amz-content-sha256", payload_hash)
//             .header("x-amz-date", datetime)
//             .send()
//             .await
//             .map_err(|e| {
//                 tracing::error!("Delete error: {e:?}");
//                 AppError::ServerError
//             })?;

//         if !response.status().is_success() {
//             let status = response.status();
//             let body = response.text().await.unwrap_or_default();
//             tracing::error!("S3 delete failed: {} - {}", status, body);
//             return Err(AppError::ServerError);
//         }

//         Ok(())
//     }
// }
