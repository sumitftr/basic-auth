use std::sync::Arc;
use util::AppError;

impl crate::Db {
    pub async fn upload_icon(
        self: &Arc<Self>,
        data: axum::body::Bytes,
        mut filename: String,
        _id: &str,
    ) -> Result<String, AppError> {
        // checking if the user sent icon is valid or not
        let content_type = util::validation::is_icon_valid(&mut filename, &data)?;
        filename = format!("icon/{_id}-{filename}");
        self.upload_image(data, &filename, &content_type).await
    }

    pub async fn upload_banner(
        self: &Arc<Self>,
        data: axum::body::Bytes,
        mut filename: String,
        _id: &str,
    ) -> Result<String, AppError> {
        // checking if the user sent banner is valid or not
        let content_type = util::validation::is_banner_valid(&mut filename, &data)?;
        filename = format!("banner/{_id}-{filename}");
        self.upload_image(data, &filename, &content_type).await
    }
}
