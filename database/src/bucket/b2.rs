use common::AppError;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct BlackBlazeB2 {
    pub client: reqwest::Client,
    pub account_id: String,
    pub application_key: String,
    pub endpoint: String,
    pub name: String,
    pub bucket_id: String,
    pub api_url: RwLock<String>, // Changed to RwLock to update after auth
    pub auth_token: RwLock<Option<String>>,
    pub public_url: String,
}

#[derive(Debug, Deserialize)]
struct AuthorizeAccountResponse {
    #[serde(rename = "authorizationToken")]
    authorization_token: String,
    #[serde(rename = "apiUrl")]
    api_url: String,
}

#[derive(Debug, Serialize)]
struct GetUploadUrlRequest {
    #[serde(rename = "bucketId")]
    bucket_id: String,
}

#[derive(Debug, Deserialize)]
struct GetUploadUrlResponse {
    #[serde(rename = "uploadUrl")]
    upload_url: String,
    #[serde(rename = "authorizationToken")]
    authorization_token: String,
}

impl Default for BlackBlazeB2 {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            account_id: std::env::var("BUCKET_ACCESS_KEY").unwrap(),
            application_key: std::env::var("BUCKET_SECRET_KEY").unwrap(),
            endpoint: std::env::var("BUCKET_ENDPOINT").unwrap(),
            auth_token: RwLock::new(None),
            api_url: RwLock::new(std::env::var("BUCKET_ENDPOINT").unwrap()),
            name: std::env::var("BUCKET_NAME").unwrap(),
            bucket_id: std::env::var("BUCKET_ID").unwrap(),
            public_url: std::env::var("BUCKET_PUBLIC_URL").unwrap(),
        }
    }
}

impl BlackBlazeB2 {
    async fn get_auth_token(&self) -> Result<String, AppError> {
        // Check if we have a cached token
        {
            let token = self.auth_token.read().await;
            if let Some(t) = token.as_ref() {
                return Ok(t.clone());
            }
        }

        // Authorize account using the base API URL
        let auth_string = format!("{}:{}", self.account_id, self.application_key);
        let auth_header = format!(
            "Basic {}",
            base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                auth_string.as_bytes()
            )
        );

        let response = self
            .client
            .get(format!("{}/b2api/v2/b2_authorize_account", self.endpoint))
            .header("Authorization", auth_header)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to authorize account: {e:#?}");
                AppError::ServerError
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Authorization failed: {}", error_text);
            return Err(AppError::ServerError);
        }

        let auth_response: AuthorizeAccountResponse = response.json().await.map_err(|e| {
            tracing::error!("Failed to parse auth response: {e:#?}");
            AppError::ServerError
        })?;

        // Update the API URL with the one returned from authorization
        {
            let mut api_url = self.api_url.write().await;
            *api_url = auth_response.api_url.clone();
            tracing::info!("Updated API URL to: {}", auth_response.api_url);
        }

        // Cache the token
        {
            let mut token = self.auth_token.write().await;
            *token = Some(auth_response.authorization_token.clone());
        }

        Ok(auth_response.authorization_token)
    }

    async fn get_upload_url(&self) -> Result<GetUploadUrlResponse, AppError> {
        let auth_token = self.get_auth_token().await?;
        let api_url = self.api_url.read().await.clone();

        let response = self
            .client
            .post(format!("{}/b2api/v2/b2_get_upload_url", api_url))
            .header("Authorization", &auth_token)
            .json(&GetUploadUrlRequest { bucket_id: self.bucket_id.clone() })
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get upload URL: {e:#?}");
                AppError::ServerError
            })?;

        if !response.status().is_success() {
            let response_status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Get upload URL failed: {}", error_text);

            // If unauthorized, clear the cached token and API URL
            if response_status == 401 {
                let mut token = self.auth_token.write().await;
                *token = None;
            }

            return Err(AppError::ServerError);
        }

        response.json().await.map_err(|e| {
            tracing::error!("Failed to parse upload URL response: {e:#?}");
            AppError::ServerError
        })
    }
}

impl crate::Db {
    pub async fn upload_image(
        self: &Arc<Self>,
        data: axum::body::Bytes,
        filename: &str,
        content_type: &str,
    ) -> Result<String, AppError> {
        if data.is_empty() {
            tracing::error!("Empty file data for: {}", filename);
            return Err(AppError::ServerError);
        }

        // Calculate SHA1 hash
        let mut hasher = Sha1::new();
        hasher.update(&data);
        let sha1_hash = format!("{:x}", hasher.finalize());

        // Get upload URL
        let upload_info = self.bucket.get_upload_url().await?;

        // URL encode the filename
        let encoded_filename = urlencoding::encode(filename);

        // Upload file
        let response = self
            .bucket
            .client
            .post(&upload_info.upload_url)
            .header("Authorization", &upload_info.authorization_token)
            .header("Content-Type", content_type)
            .header("Content-Length", data.len())
            .header("X-Bz-File-Name", encoded_filename.as_ref())
            .header("X-Bz-Content-Sha1", &sha1_hash)
            .body(data.to_vec())
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Upload failed for {}: {e:#?}", filename);
                AppError::ServerError
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Upload error for {}: {}", filename, error_text);
            return Err(AppError::ServerError);
        }

        Ok(format!("{}/{}", self.bucket.public_url, filename))
    }

    pub async fn delete_image(self: &Arc<Self>, filename: &str) -> Result<(), AppError> {
        let auth_token = self.bucket.get_auth_token().await?;
        let api_url = self.bucket.api_url.read().await.clone();

        // First, get file info to get the file ID
        let list_response = self
            .bucket
            .client
            .post(format!("{}/b2api/v2/b2_list_file_names", api_url))
            .header("Authorization", &auth_token)
            .json(&serde_json::json!({
                "bucketId": self.bucket.bucket_id,
                "startFileName": filename,
                "maxFileCount": 1,
                "prefix": filename
            }))
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to list file: {e:#?}");
                AppError::ServerError
            })?;

        if !list_response.status().is_success() {
            let error_text = list_response.text().await.unwrap_or_default();
            tracing::error!("List file failed: {}", error_text);
            return Err(AppError::ServerError);
        }

        #[derive(Deserialize)]
        struct FileInfo {
            #[serde(rename = "fileId")]
            file_id: String,
            #[serde(rename = "fileName")]
            file_name: String,
        }

        #[derive(Deserialize)]
        struct ListFilesResponse {
            files: Vec<FileInfo>,
        }

        let list_result: ListFilesResponse = list_response.json().await.map_err(|e| {
            tracing::error!("Failed to parse list response: {e:#?}");
            AppError::ServerError
        })?;

        let file_info =
            list_result.files.into_iter().find(|f| f.file_name == filename).ok_or_else(|| {
                tracing::error!("File not found: {}", filename);
                AppError::ServerError
            })?;

        // Delete the file
        let delete_response = self
            .bucket
            .client
            .post(format!("{}/b2api/v2/b2_delete_file_version", api_url))
            .header("Authorization", &auth_token)
            .json(&serde_json::json!({
                "fileId": file_info.file_id,
                "fileName": filename
            }))
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete file: {e:#?}");
                AppError::ServerError
            })?;

        if !delete_response.status().is_success() {
            let error_text = delete_response.text().await.unwrap_or_default();
            tracing::error!("Delete failed: {}", error_text);
            return Err(AppError::ServerError);
        }

        Ok(())
    }
}
