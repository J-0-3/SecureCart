//! Logic for storing and operating on stored media objects, such as images.
use std::path::PathBuf;
#[expect(clippy::useless_attribute, reason = "This is from clippy::restricted")]
#[expect(
    clippy::std_instead_of_alloc,
    reason = "Alloc is not available outside of no_std"
)]
use std::sync::Arc;

use object_store::{path::Path, Attribute, Attributes, ObjectStore, PutOptions, PutPayload};
use sha2::{Digest as _, Sha256};

/// The prefix within the storage bucket under which images will be stored.
const IMAGE_PREFIX: &str = "/images";

/// Supported image file types.
enum ImageFileType {
    /// A PNG image
    Png,
    /// A JPEG image
    Jpg,
    /// A GIF image
    Gif,
}

impl ImageFileType {
    /// Get the file type from the file's magic bytes, returns None if the
    /// file does not match any of the supported types.
    const fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            &[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, ..] => Some(Self::Png),
            &[0xff, 0xd8, 0xff, 0xe0 | 0xee, ..]
            | &[0xff, 0xd8, 0xff, 0xe1, _, _, 0x45, 0x78, 0x69, 0x66, 0, 0, ..] => Some(Self::Jpg),
            &[0x47, 0x49, 0x46, 0x38, 0x37 | 0x39, 0x61, ..] => Some(Self::Gif),
            _ => None,
        }
    }
    /// Get the file extension typically associated with this file type.
    const fn extension(&self) -> &str {
        match *self {
            Self::Png => "png",
            Self::Jpg => "jpg",
            Self::Gif => "gif",
        }
    }
    /// Get the mimetype associated with this file type.
    const fn mimetype(&self) -> &str {
        match *self {
            Self::Png => "image/png",
            Self::Jpg => "image/jpeg",
            Self::Gif => "image/gif",
        }
    }
}

/// Store an image in the media store. Will return the path under the storage bucket
/// at which the image has been stored, and will error if the image file is of an
/// unsupported type or the networked storage access fails.
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
    let mut object_attributes = Attributes::with_capacity(1);
    object_attributes.insert(
        Attribute::ContentType,
        file_type.mimetype().to_owned().into(),
    );
    object_attributes.insert(Attribute::ContentDisposition, "inline".into());
    let put_opts = PutOptions {
        attributes: object_attributes,
        ..Default::default()
    };
    store
        .put_opts(
            &Path::from(object_path.as_str()),
            PutPayload::from(image),
            put_opts,
        )
        .await
        .map_err(errors::StorageError::from)?;
    Ok(object_path)
}

/// Errors returned from this module.
pub mod errors {
    use thiserror::Error;
    /// Errors returned when storing an image.
    #[derive(Debug, Error)]
    pub enum StoreImageError {
        /// The image file was not identified as being a supported type
        /// (see ``ImageFileType``).
        #[error("Image is of invalid file type")]
        InvalidFileType,
        /// An error occurred during the actual storage operation.
        #[error(transparent)]
        StorageError(#[from] StorageError),
    }

    /// An error passed up from the underlying object store.
    #[derive(Debug, Error)]
    #[error(transparent)]
    pub struct StorageError(#[from] object_store::Error);
}
