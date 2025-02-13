use std::path::PathBuf;
#[expect(clippy::useless_attribute, reason = "This is from clippy::restricted")]
#[expect(
    clippy::std_instead_of_alloc,
    reason = "Alloc is not available outside of no_std"
)]
use std::sync::Arc;

use object_store::{path::Path, ObjectStore, PutPayload};
use sha2::{Digest as _, Sha256};

const IMAGE_PREFIX: &str = "/images";

enum ImageFileType {
    PNG,
    JPG,
    GIF,
}

impl ImageFileType {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, ..] => Some(Self::PNG),
            [0xff, 0xd8, 0xff, 0xe0 | 0xee, ..]
            | [0xff, 0xd8, 0xff, 0xe1, _, _, 0x45, 0x78, 0x69, 0x66, 0, 0] => Some(Self::JPG),
            [0x47, 0x49, 0x46, 0x38, 0x37 | 0x39, 0x61] => Some(Self::GIF),
            _ => None,
        }
    }
    fn extension(&self) -> &str {
        match self {
            Self::PNG => "png",
            Self::JPG => "jpg",
            Self::GIF => "gif",
        }
    }
}

pub async fn store_image(
    store: Arc<dyn ObjectStore>,
    image: Vec<u8>,
) -> Result<String, errors::StoreImageError> {
    let mut hasher = Sha256::new();
    hasher.update(&image);
    let hash = hasher.finalize();
    let file_type =
        ImageFileType::from_bytes(&image).ok_or(errors::StoreImageError::InvalidFileType)?;
    let object_name = format!("{hash:x}");
    let object_path = PathBuf::new()
        .join(IMAGE_PREFIX)
        .join(object_name)
        .with_extension(file_type.extension())
        .to_string_lossy()
        .into_owned();
    // object_store will upsert by default, and since we use hashes, this will implicitely
    // dedup image storage.
    store
        .put(&Path::from(object_path.as_str()), PutPayload::from(image))
        .await
        .map_err(errors::StorageError::from)?;
    Ok(object_path)
}

pub mod errors {
    use thiserror::Error;
    #[derive(Debug, Error)]
    pub enum StoreImageError {
        #[error("Image is of invalid file type")]
        InvalidFileType,
        #[error(transparent)]
        StorageError(#[from] StorageError),
    }

    #[derive(Debug, Error)]
    #[error(transparent)]
    pub struct StorageError(#[from] object_store::Error);
}
