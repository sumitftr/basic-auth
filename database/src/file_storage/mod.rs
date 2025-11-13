use common::AppError;
use std::sync::Arc;

pub struct FileStorage {
    pub client: s3::Client,
    pub name: String,
    pub public_url: String,
}

impl Default for FileStorage {
    fn default() -> Self {
        let credentials = s3::config::Credentials::new(
            std::env::var("BUCKET_ACCESS_KEY").unwrap(),
            std::env::var("BUCKET_SECRET_KEY").unwrap(),
            None,
            None,
            "generic",
        );

        let config = s3::Config::builder()
            .endpoint_url(std::env::var("BUCKET_ENDPOINT").unwrap())
            .credentials_provider(credentials)
            .region(s3::config::Region::new(
                std::env::var("BUCKET_REGION").unwrap(),
            ))
            .build();

        let client = s3::Client::from_conf(config);

        Self {
            client,
            name: std::env::var("BUCKET_NAME").unwrap(),
            public_url: std::env::var("BUCKET_PUBLIC_URL").unwrap(),
        }
    }
}

impl crate::Db {
    pub async fn upload_image(
        self: &Arc<Self>,
        filename: &str,
        data: axum::body::Bytes,
        content_type: &str,
    ) -> Result<String, AppError> {
        self.bucket
            .client
            .put_object()
            .bucket(&self.bucket.name)
            .key(filename)
            .body(data.into())
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("{e:?}");
                AppError::ServerError
            })?;

        Ok(format!("{}/{}", self.bucket.public_url, filename))
    }

    pub async fn delete_image(self: &Arc<Self>, filename: &str) -> Result<(), AppError> {
        self.bucket
            .client
            .delete_object()
            .bucket(&self.bucket.name)
            .key(filename)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("{e:?}");
                AppError::ServerError
            })?;

        Ok(())
    }
}
