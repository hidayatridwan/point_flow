use anyhow::{Context, Result};
use aws_sdk_s3::primitives::ByteStream;

use crate::config::S3Settings;

/// Maksimal ukuran file gambar yang diterima (foto produk, QRIS).
pub const MAX_IMAGE_BYTES: usize = 2 * 1024 * 1024;

/// Ekstensi file yang diizinkan berdasarkan content type multipart.
/// Ekstensi diambil dari content type, bukan nama file dari browser.
pub fn image_extension(content_type: &str) -> Option<&'static str> {
    match content_type.to_ascii_lowercase().as_str() {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/webp" => Some("webp"),
        _ => None,
    }
}

/// Object storage S3-compatible (foto produk, gambar QRIS).
/// Database hanya menyimpan object key; URL publik disusun dari `public_url`.
pub struct ObjectStorage {
    client: aws_sdk_s3::Client,
    bucket: String,
    public_url: String,
}

impl ObjectStorage {
    pub async fn from_settings(settings: &S3Settings) -> Result<Self> {
        let credentials = aws_credential_types::Credentials::new(
            settings.access_key_id.clone(),
            settings.secret_access_key.clone(),
            None,
            None,
            "s3-env",
        );
        let sdk_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .endpoint_url(settings.endpoint_url.clone())
            .region(aws_config::Region::new(settings.region.clone()))
            .credentials_provider(credentials)
            .load()
            .await;
        let s3_config = aws_sdk_s3::config::Builder::from(&sdk_config)
            .force_path_style(settings.force_path_style)
            .build();
        Ok(Self {
            client: aws_sdk_s3::Client::from_conf(s3_config),
            bucket: settings.bucket.clone(),
            public_url: settings.public_url.clone(),
        })
    }

    pub async fn put_object(&self, key: &str, bytes: Vec<u8>, content_type: &str) -> Result<()> {
        // Key selalu unik per upload (UUID), jadi aman di-cache browser/Tigris
        // edge setahun penuh: hemat Tigris Class B request dan kunjungan
        // ulang menampilkan gambar instan dari cache.
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(bytes))
            .content_type(content_type.to_string())
            .cache_control("public, max-age=31536000, immutable")
            .send()
            .await
            .with_context(|| format!("failed to upload object {key}"))?;
        Ok(())
    }

    pub fn public_url(&self, key: &str) -> String {
        format!("{}/{}", self.base_url(), key)
    }

    pub fn base_url(&self) -> &str {
        self.public_url.trim_end_matches('/')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_extension_accepts_supported_types() {
        assert_eq!(image_extension("image/jpeg"), Some("jpg"));
        assert_eq!(image_extension("image/png"), Some("png"));
        assert_eq!(image_extension("image/webp"), Some("webp"));
        assert_eq!(image_extension("IMAGE/JPEG"), Some("jpg"));
    }

    #[test]
    fn image_extension_rejects_other_types() {
        assert_eq!(image_extension("image/svg+xml"), None);
        assert_eq!(image_extension("application/pdf"), None);
        assert_eq!(image_extension(""), None);
    }
}
