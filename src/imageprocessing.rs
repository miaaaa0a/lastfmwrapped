use std::error::Error;
use image::{DynamicImage, ImageBuffer, ImageReader, Rgba};
use imageproc::drawing::{draw_text, text_size};
use ab_glyph::{FontRef, PxScale};

pub fn calculate_text_centre(img: &DynamicImage, scale: PxScale, font: &FontRef, text: &str) -> (i32, i32) {
    let text_size = text_size(scale, font, text);
    let image_size = (img.width(), img.height());
    let txtx: i32 = ((image_size.0 / 2) - (text_size.0 / 2)).try_into().unwrap_or(0);
    let txty: i32 = ((image_size.1 / 2) - (text_size.1 / 2)).try_into().unwrap_or(0);

    (txtx, txty)
}


pub fn minutes_listened(text: String) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
    let img = ImageReader::open("imgs/minuteslistened.png")?.decode()?;
    let font = FontRef::try_from_slice(include_bytes!("../fonts/SpotifyMix-Extrabold.ttf"))?;
    let scale = PxScale::from(290.0);

    let txtc = calculate_text_centre(&img, scale, &font, &text);

    // in official image text is offset from bottom by 378 / 2 px
    let modified_img = draw_text(&img, Rgba([255, 255, 255, 255]), txtc.0, txtc.1 - 189, scale, &font, &text);
    Ok(modified_img)
}