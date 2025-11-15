use crate::AppError;

fn detect_image_format(data: &axum::body::Bytes) -> Result<String, AppError> {
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Ok("jpg".to_string());
    }

    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Ok("png".to_string());
    }

    if data.len() >= 12
        && data.starts_with(&[0x52, 0x49, 0x46, 0x46])
        && data[8..12] == [0x57, 0x45, 0x42, 0x50]
    {
        return Ok("webp".to_string());
    }

    // if data.starts_with(&[0x47, 0x49, 0x46, 0x38])
    //     && data.len() >= 6
    //     && (data[4] == 0x37 || data[4] == 0x39)
    //     && data[5] == 0x61 {
    //     return Ok("image/gif".to_string());
    // }

    Err(AppError::InvalidData(
        "Invalid image type: Only jpg, jpeg, png, webp are allowed.",
    ))
}

pub fn is_icon_valid(filepath: &mut String, data: &axum::body::Bytes) -> Result<String, AppError> {
    // validate file size (max 5MiB)
    if data.len() > 5 * 1024 * 1024 {
        return Err(AppError::InvalidData("File too large (max 5MiB)"));
    }

    let img_type = detect_image_format(data)?;

    let mut filename = filepath.split('/').next_back().unwrap().to_string();
    // remove existing extension if present
    if let Some(dot_pos) = filename.rfind('.') {
        filename.truncate(dot_pos);
    }
    // add the correct extension
    filename.push('.');
    filename.push_str(&img_type);
    // replace the original string with new string
    let _ = std::mem::replace(filepath, filename);

    Ok(format!("image/{img_type}"))
}
