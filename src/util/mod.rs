use std::{path::PathBuf, time::Duration};

use image::{Delay, DynamicImage, Frame, ImageBuffer, Luma, LumaA, Primitive, Rgb, Rgba};

use crate::app::op_queue::Output;

pub mod extensions;

#[macro_export]
macro_rules! min {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = min!($($z),*);
        if $x < y {
            $x
        } else {
            y
        }
    }}
}

#[macro_export]
macro_rules! max {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = max!($($z),*);
        if $x > y {
            $x
        } else {
            y
        }
    }}
}

#[derive(Clone, Debug)]
pub struct Image {
    pub image: DynamicImage,
    pub delay: Duration,
}

impl Image {
    pub fn new(image: DynamicImage) -> Self {
        Image {
            image,
            delay: Duration::default(),
        }
    }

    pub fn with_delay(image: DynamicImage, delay: Duration) -> Self {
        Image { image, delay }
    }

    pub fn buffer(&self) -> &DynamicImage {
        &self.image
    }

    pub fn buffer_mut(&mut self) -> &mut DynamicImage {
        &mut self.image
    }
}

impl From<ImageBuffer<Rgba<u8>, Vec<u8>>> for Image {
    fn from(buffer: ImageBuffer<Rgba<u8>, Vec<u8>>) -> Self {
        Image {
            image: DynamicImage::ImageRgba8(buffer),
            delay: Duration::default(),
        }
    }
}

impl From<Frame> for Image {
    fn from(frame: Frame) -> Self {
        let (num, deno) = frame.delay().numer_denom_ms();
        let delay = Duration::from_millis((num / deno) as u64);
        let buffer = frame.into_buffer();
        Image {
            image: DynamicImage::ImageRgba8(buffer),
            delay,
        }
    }
}

impl From<Image> for Frame {
    fn from(image: Image) -> Frame {
        let duration = image.delay;
        let frame = image.image.to_rgba8();
        Frame::from_parts(frame, 0, 0, Delay::from_saturating_duration(duration))
    }
}

pub enum UserEvent {
    ErrorMessage(String),
    QueueLoad(PathBuf),
    QueueSave(PathBuf),
    QueueDelete(PathBuf),
    Output(Option<Output>),
    Exit,
}

#[derive(Debug, Clone)]
pub struct ImageData {
    pub frames: Vec<Image>,
    pub metadata: Vec<(String, String)>,
}

impl ImageData {
    pub fn new(frames: Vec<Image>, metadata: Vec<(String, String)>) -> Self {
        Self { frames, metadata }
    }
}

impl From<Vec<Image>> for ImageData {
    fn from(frames: Vec<Image>) -> Self {
        Self {
            frames,
            metadata: Vec::new(),
        }
    }
}

pub trait HasAlpha {
    fn has_alpha(&self) -> bool;
}

impl HasAlpha for DynamicImage {
    fn has_alpha(&self) -> bool {
        match self {
            DynamicImage::ImageLuma8(b) => b.has_alpha(),
            DynamicImage::ImageLumaA8(b) => b.has_alpha(),
            DynamicImage::ImageRgb8(b) => b.has_alpha(),
            DynamicImage::ImageRgba8(b) => b.has_alpha(),
            DynamicImage::ImageLuma16(b) => b.has_alpha(),
            DynamicImage::ImageLumaA16(b) => b.has_alpha(),
            DynamicImage::ImageRgb16(b) => b.has_alpha(),
            DynamicImage::ImageRgba16(b) => b.has_alpha(),
            DynamicImage::ImageRgb32F(b) => b.has_alpha(),
            DynamicImage::ImageRgba32F(b) => b.has_alpha(),
            _ => panic!("Unknown color space name. This is a bug."),
        }
    }
}

pub enum ColorBits {
    U16,
    U8,
    F32,
}

pub trait GetColorBits {
    fn get_color_bits(&self) -> ColorBits;
}

impl GetColorBits for DynamicImage {
    fn get_color_bits(&self) -> ColorBits {
        use ColorBits::*;
        match self {
            DynamicImage::ImageLuma8(_) => U8,
            DynamicImage::ImageLumaA8(_) => U8,
            DynamicImage::ImageRgb8(_) => U8,
            DynamicImage::ImageRgba8(_) => U8,
            DynamicImage::ImageLuma16(_) => U16,
            DynamicImage::ImageLumaA16(_) => U16,
            DynamicImage::ImageRgb16(_) => U16,
            DynamicImage::ImageRgba16(_) => U16,
            DynamicImage::ImageRgb32F(_) => F32,
            DynamicImage::ImageRgba32F(_) => F32,
            _ => panic!("Unknown color space name. This is a bug."),
        }
    }
}

impl<T: Primitive> HasAlpha for ImageBuffer<Luma<T>, Vec<T>> {
    fn has_alpha(&self) -> bool {
        false
    }
}

impl<T: Primitive> HasAlpha for ImageBuffer<LumaA<T>, Vec<T>> {
    fn has_alpha(&self) -> bool {
        true
    }
}

// TODO switch to generics when the image crate fixes its shit.
impl HasAlpha for ImageBuffer<Rgb<u8>, Vec<u8>> {
    fn has_alpha(&self) -> bool {
        false
    }
}

impl HasAlpha for ImageBuffer<Rgb<u16>, Vec<u16>> {
    fn has_alpha(&self) -> bool {
        false
    }
}

impl HasAlpha for ImageBuffer<Rgb<f32>, Vec<f32>> {
    fn has_alpha(&self) -> bool {
        false
    }
}

impl HasAlpha for ImageBuffer<Rgba<u8>, Vec<u8>> {
    fn has_alpha(&self) -> bool {
        true
    }
}

impl HasAlpha for ImageBuffer<Rgba<u16>, Vec<u16>> {
    fn has_alpha(&self) -> bool {
        true
    }
}

impl HasAlpha for ImageBuffer<Rgba<f32>, Vec<f32>> {
    fn has_alpha(&self) -> bool {
        true
    }
}
