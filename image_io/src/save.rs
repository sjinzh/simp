use std::{
    error, fmt,
    fs::{rename, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use image::{
    codecs::{gif::GifEncoder, tiff::TiffEncoder},
    Frame, GenericImageView, ImageError, ImageOutputFormat,
};
use libwebp::WebPEncodeLosslessRGBA;
use util::Image;
use uuid::Uuid;
use webp_animation::{Encoder, EncoderOptions, EncodingConfig};

type SaveResult<T> = Result<T, SaveError>;

#[derive(Debug)]
pub enum SaveError {
    Image(ImageError),
    Io(std::io::Error),
    WebpAnimation(webp_animation::Error),
    LibWebp(libwebp::error::WebPSimpleError),
}

impl fmt::Display for SaveError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SaveError::Image(ref e) => e.fmt(f),
            SaveError::Io(ref e) => e.fmt(f),
            SaveError::WebpAnimation(_) => write!(f, "error encoding webp"),
            SaveError::LibWebp(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for SaveError {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            SaveError::Image(ref e) => Some(e),
            SaveError::Io(ref e) => Some(e),
            SaveError::WebpAnimation(_) => None,
            SaveError::LibWebp(ref e) => Some(e),
        }
    }
}

impl From<ImageError> for SaveError {
    #[inline]
    fn from(err: ImageError) -> SaveError {
        SaveError::Image(err)
    }
}

impl From<std::io::Error> for SaveError {
    #[inline]
    fn from(err: std::io::Error) -> SaveError {
        SaveError::Io(err)
    }
}

impl From<webp_animation::Error> for SaveError {
    #[inline]
    fn from(err: webp_animation::Error) -> SaveError {
        SaveError::WebpAnimation(err)
    }
}

impl From<libwebp::error::WebPSimpleError> for SaveError {
    #[inline]
    fn from(err: libwebp::error::WebPSimpleError) -> SaveError {
        SaveError::LibWebp(err)
    }
}

fn open_file(path: impl AsRef<Path>) -> Result<File, std::io::Error> {
    OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
}

fn get_temp_path(path: impl AsRef<Path>) -> PathBuf {
    let uuid = Uuid::new_v4().to_string();
    let mut buf = path.as_ref().to_path_buf();
    buf.set_file_name(uuid);
    buf
}

#[inline]
pub fn save_with_format(
    path: impl AsRef<Path>,
    image: &Image,
    format: ImageOutputFormat,
) -> SaveResult<()> {
    let temp_path = get_temp_path(path.as_ref());
    let mut file = open_file(&temp_path)?;
    image.buffer().write_to(&mut file, format)?;

    Ok(rename(temp_path, path)?)
}

#[inline]
pub fn tiff(path: impl AsRef<Path>, image: &Image) -> SaveResult<()> {
    let temp_path = get_temp_path(path.as_ref());
    let file = open_file(&temp_path)?;

    let encoder = TiffEncoder::new(file);
    let buffer = image.buffer();

    encoder.encode(
        buffer.as_bytes(),
        buffer.width(),
        buffer.height(),
        buffer.color(),
    )?;

    Ok(rename(temp_path, path)?)
}

#[inline]
pub fn gif(path: impl AsRef<Path>, images: Vec<Image>) -> SaveResult<()> {
    let temp_path = get_temp_path(path.as_ref());
    let file = open_file(&temp_path)?;

    let frames: Vec<Frame> = images.into_iter().map(|image| image.into()).collect();
    let mut encoder = GifEncoder::new(file);
    encoder.encode_frames(frames)?;

    Ok(rename(temp_path, path)?)
}

#[inline]
pub fn webp_animation(path: impl AsRef<Path>, images: Vec<Image>) -> SaveResult<()> {
    let config = EncodingConfig {
        encoding_type: webp_animation::prelude::EncodingType::Lossless,
        quality: 100.0,
        method: 6,
    };
    let dimensions = images[0].buffer().dimensions();
    let options = EncoderOptions {
        encoding_config: Some(config),
        ..Default::default()
    };
    let mut encoder = Encoder::new_with_options(dimensions, options)?;
    let mut timestamp: i32 = 0;
    for image in images {
        encoder.add_frame(&image.buffer().to_rgba8().into_raw(), timestamp)?;
        timestamp += image.delay.as_millis() as i32;
    }

    let webp_data = encoder.finalize(timestamp)?;

    let temp_path = get_temp_path(path.as_ref());
    let mut file = open_file(&temp_path)?;
    file.write_all(&*webp_data)?;

    Ok(rename(temp_path, path)?)
}

#[inline]
pub fn webp(path: impl AsRef<Path>, image: &Image) -> SaveResult<()> {
    let (width, height) = image.buffer().dimensions();
    let webp_data = WebPEncodeLosslessRGBA(
        &image.buffer().to_rgba8().into_raw(),
        width,
        height,
        width * 4,
    )?;

    let temp_path = get_temp_path(path.as_ref());
    let mut file = open_file(&temp_path)?;
    file.write_all(&*webp_data)?;

    Ok(rename(temp_path, path)?)
}
