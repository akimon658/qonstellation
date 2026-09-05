use image::{ImageFormat, imageops};
use reqwest::multipart::{Form, Part};
use std::io::Cursor;

use crate::app_config::config::Config;

const MAX_PIXELS: f32 = 2560.0 * 1600.0;

/// Resized image encoded as WebP with its MIME type.
/// WebP encoding is not expected to fail; errors are propagated so the caller
/// can retry the whole event instead of posting an incomplete message.
pub fn resize_image(image_data: &[u8]) -> anyhow::Result<(Vec<u8>, &'static str, &'static str)> {
    let img = image::load_from_memory(image_data)?;
    let (width, height) = (img.width() as f32, img.height() as f32);

    let pixels = width * height;
    let resized = if pixels > MAX_PIXELS {
        let ratio = (MAX_PIXELS / pixels).sqrt();
        let new_width = ((width * ratio) as u32).max(1);
        let new_height = ((height * ratio) as u32).max(1);
        imageops::resize(&img, new_width, new_height, imageops::FilterType::Lanczos3)
    } else {
        img.to_rgba8()
    };

    let mut output = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(resized).write_to(&mut output, ImageFormat::WebP)?;

    Ok((output.into_inner(), "image.webp", "image/webp"))
}

pub async fn upload_file(
    http_client: &reqwest::Client,
    config: &Config,
    channel_id: &str,
    data: &[u8],
    filename: &str,
    content_type: &str,
    access_token: &str,
) -> anyhow::Result<String> {
    let url = format!("{}/api/v3/files", config.traq_base_url);

    let part = Part::bytes(data.to_vec())
        .file_name(filename.to_string())
        .mime_str(content_type)?;

    let form = Form::new()
        .text("channelId", channel_id.to_string())
        .part("file", part);

    let response = http_client
        .post(&url)
        .bearer_auth(access_token)
        .multipart(form)
        .send()
        .await?;

    if !response.status().is_success() {
        let text = response.text().await?;
        return Err(anyhow::anyhow!("Failed to upload file: {}", text));
    }

    let json: serde_json::Value = response.json().await?;
    let file_id = json["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing file ID in response"))?;

    Ok(file_id.to_string())
}
