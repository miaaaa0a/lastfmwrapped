use std::error::Error;
use image::{imageops::{self, FilterType}, DynamicImage, ImageBuffer, ImageReader, Rgba};
use imageproc::drawing::{draw_text, text_size};
use ab_glyph::{FontRef, PxScale};
use chrono::{TimeZone, Utc};
use thousands::Separable;

struct SpotifyFont<'a> {
    regular: FontRef<'a>,
    medium: FontRef<'a>,
    extra_bold: FontRef<'a>,
}

impl<'b> SpotifyFont<'b> {
    pub fn new(fonts: Vec<FontRef<'b>>) -> Self {
        Self {
            regular: fonts[0].clone(),
            medium: fonts[1].clone(),
            extra_bold: fonts[2].clone(),
        }
    }
}

fn calculate_text_centre(img: &DynamicImage, scale: PxScale, font: &FontRef, text: &str) -> (i32, i32) {
    let text_size = text_size(scale, font, text);
    let image_size = (img.width(), img.height());
    let txtx: i32 = ((image_size.0 / 2) - (text_size.0 / 2)).try_into().unwrap_or(0);
    let txty: i32 = ((image_size.1 / 2) - (text_size.1 / 2)).try_into().unwrap_or(0);

    (txtx, txty)
}

fn batch_draw_text(strs: Vec<&str>, xys: Vec<(i32, i32)>, img: ImageBuffer<Rgba<u8>, Vec<u8>>, scale: PxScale, fonts: Vec<&FontRef>, offset: (i32, i32)) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut mimg = img.clone();
    for i in 0..strs.len() {
        mimg = draw_text(
            &mimg, 
            Rgba([255, 255, 255, 255]), 
            xys[i].0 + offset.0, 
            xys[i].1 + offset.1, 
            scale, 
            fonts[i], 
            strs[i]
        );
    }
    mimg
}

fn fonts() -> Result<SpotifyFont<'static>, Box<dyn Error>> {
    let sf = SpotifyFont::new(
        vec![
            FontRef::try_from_slice(include_bytes!("../fonts/SpotifyMix-Regular.ttf"))?,
            FontRef::try_from_slice(include_bytes!("../fonts/SpotifyMix-Medium.ttf"))?,
            FontRef::try_from_slice(include_bytes!("../fonts/SpotifyMix-Extrabold.ttf"))?,
        ]
    );
    Ok(sf)
}

// if theres any god on this earth,
// will you please forgive me for this
// unholy piece of code?
pub fn minutes_listened(total: i64, busiest_day: i64, busiest_time: i64) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
    let img = ImageReader::open("imgs/minuteslistened.png")?.decode()?;
    let fonts = fonts().unwrap();
    let totalscale = PxScale::from(290.0);
    let busiestscale = PxScale::from(50.0);
    let busiest_day_string = Utc.timestamp_opt(busiest_day, 0).unwrap().format("%B %_d").to_string();
    let total_str = total.separate_with_commas();

    let totalc = calculate_text_centre(
        &img, 
        totalscale, 
        &fonts.extra_bold, 
        &total_str
    );
    let busiestc = calculate_text_centre(
        &img, 
        busiestscale, 
        &fonts.medium, 
        &format!("Biggest listening day: {} with {} minutes", busiest_day_string, busiest_time)
    );
    let busiest_day_len = text_size(busiestscale, &fonts.extra_bold, &busiest_day_string);
    let busiest_time_len = text_size(busiestscale, &fonts.extra_bold, &busiest_time.to_string());
    let busiest_str_len = (
        text_size(busiestscale, &fonts.medium, "Biggest listening day: "),
        text_size(busiestscale, &fonts.medium, " with "),
    );

    let busiest_day_coords = (busiestc.0 + (busiest_str_len.0.0 as i32), busiestc.1);
    let busiest_with_coords = (busiest_day_coords.0 + (busiest_day_len.0 as i32), busiest_day_coords.1);
    let busiest_time_coords = (busiest_with_coords.0 + (busiest_str_len.1.0 as i32), busiest_with_coords.1);
    let busiest_min_coords = (busiest_time_coords.0 + (busiest_time_len.0 as i32), busiest_time_coords.1);

    // draw total minutes
    // in official image text is offset from bottom by 378 / 2 px
    let mut modified_img = draw_text(
        &img, 
        Rgba([255, 255, 255, 255]), 
        totalc.0, 
        totalc.1 - 189, 
        totalscale, 
        &fonts.extra_bold, 
        &total_str
    );
    modified_img = batch_draw_text(
        vec![
            "Biggest listening day: ",
            &busiest_day_string,
            " with ",
            &busiest_time.to_string(),
            " minutes"
        ], 
        vec![
            busiestc,
            busiest_day_coords,
            busiest_with_coords,
            busiest_time_coords,
            busiest_min_coords
        ], 
        modified_img, 
        busiestscale, 
        vec![
            &fonts.medium,
            &fonts.extra_bold,
            &fonts.medium,
            &fonts.extra_bold,
            &fonts.medium,
        ],
        (0, 132)
    );

    Ok(modified_img)
}

pub fn top_song(name: String, scrobbles: i64, cover: DynamicImage) -> Result<DynamicImage, Box<dyn Error>> {
    let mut img = ImageReader::open("imgs/topsong.png")?.decode()?;
    let fonts = fonts();
    // my top song and artist name
    let h1scale = PxScale::from(55.0);
    // total streams
    let h2scale = PxScale::from(45.0);
    // song name
    let songscale = PxScale::from(100.0);
    // all streams
    let scrobblescale = PxScale::from(90.0);
    // song cover scale
    let coverscale = (584, 584);
    let coverxy = (248, 204);

    let scaled_cover = cover.resize(coverscale.0, coverscale.1, FilterType::CatmullRom);
    imageops::overlay(&mut img, &scaled_cover, coverxy.0, coverxy.1);
    Ok(img)
}