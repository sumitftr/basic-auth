use common::AppError;
use std::sync::Arc;

impl crate::Db {
    pub async fn upload_icon(
        self: &Arc<Self>,
        data: axum::body::Bytes,
        mut filename: String,
        _id: &str,
    ) -> Result<String, AppError> {
        // checking if the user sent icon is valid or not
        let content_type = common::validation::is_icon_valid(&mut filename, &data)?;
        filename = format!("icon/{_id}-{filename}");
        self.upload_image(data, &filename, &content_type).await
    }
}
