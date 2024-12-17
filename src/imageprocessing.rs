use std::error::Error;
use image::{imageops::{self, FilterType}, DynamicImage, ImageBuffer, ImageReader, Rgba};
use imageproc::drawing::{draw_text, draw_text_mut, text_size};
use ab_glyph::{FontRef, PxScale};
use chrono::{TimeZone, Utc};
use thousands::Separable;

#[derive(Clone)]
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

struct SongFonts<'a> {
    artist: SpotifyFont<'a>,
    name: SpotifyFont<'a>,
    number: SpotifyFont<'a>,
}

impl<'b> SongFonts<'b> {
    pub fn new(font: SpotifyFont<'b>, fallback_font: SpotifyFont<'b>, strs: &Vec<&str>) -> Self {
        Self {
            artist: if check_for_cyrillic(strs[0]) { fallback_font.clone() } else { font.clone() },
            name: if check_for_cyrillic(strs[1]) { fallback_font.clone() } else { font.clone() },
            number: font.clone()
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

fn batch_draw_text(strs: Vec<&str>, xys: Vec<(i32, i32)>, img: &mut DynamicImage, scale: PxScale, fonts: Vec<&FontRef>, offset: (i32, i32)) {
    for i in 0..strs.len() {
        draw_text_mut(
            img, 
            Rgba([255, 255, 255, 255]), 
            xys[i].0 + offset.0, 
            xys[i].1 + offset.1, 
            scale, 
            fonts[i], 
            strs[i]
        );
    }
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

fn fallback_fonts() -> Result<SpotifyFont<'static>, Box<dyn Error>> {
    let sf = SpotifyFont::new(
        vec![
            FontRef::try_from_slice(include_bytes!("../fonts/FluidSans-Light.ttf"))?,
            FontRef::try_from_slice(include_bytes!("../fonts/FluidSans-Regular.ttf"))?,
            FontRef::try_from_slice(include_bytes!("../fonts/FluidSans-Black.ttf"))?,
        ]
    );
    Ok(sf)
}

fn check_for_cyrillic(str: &str) -> bool {
    let cyrillic_chars = (0x400..0x4ff).map(|x| char::from_u32(x).unwrap()).collect::<Vec<_>>();
    let mut ret = false;
    for c in str.chars() {
        if cyrillic_chars.contains(&c) {
            ret = true;
            break;
        }
    }
    ret
}

pub fn minutes_listened(total: i64, busiest_day: i64, busiest_time: i64) -> Result<DynamicImage, Box<dyn Error>> {
    let mut img = ImageReader::open("imgs/minuteslistened.png")?.decode()?;
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
    draw_text_mut(
        &mut img, 
        Rgba([255, 255, 255, 255]), 
        totalc.0, 
        totalc.1 - 189, 
        totalscale, 
        &fonts.extra_bold, 
        &total_str
    );
    batch_draw_text(
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
        &mut img, 
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

    Ok(img)
}

pub fn top_song(name: String, scrobbles: i64, cover: DynamicImage) -> Result<DynamicImage, Box<dyn Error>> {
    let mut img = ImageReader::open("imgs/topsong.png")?.decode()?;
    let song_info = name.split(" - ").collect::<Vec<_>>();
    let fonts = SongFonts::new(fonts()?, fallback_fonts()?, &song_info);
    // artist name
    let artistscale = PxScale::from(60.0);
    // song name
    let songscale = PxScale::from(100.0);
    // all streams
    let scrobblescale = PxScale::from(85.0);
    // song cover scale
    let coverscale = (584, 584);
    let coverxy = (248, 204);

    // text centres
    // track name
    let trackc = calculate_text_centre(
        &img, 
        songscale, 
        &fonts.name.extra_bold, 
        song_info[1]
    );
    // artist name
    let artistc = calculate_text_centre(
        &img, 
        artistscale, 
        &fonts.artist.regular, 
        song_info[0]
    );
    // scrobbles
    let scrobblesc = calculate_text_centre(
        &img, 
        scrobblescale, 
        &fonts.number.regular, 
        &scrobbles.to_string()
    );

    let scaled_cover = cover.resize(coverscale.0, coverscale.1, FilterType::CatmullRom);
    imageops::overlay(&mut img, &scaled_cover, coverxy.0, coverxy.1);
    draw_text_mut(
        &mut img,
        Rgba([0, 0, 0, 255]),
        trackc.0,
        trackc.1 + 230,
        songscale,
        &fonts.name.extra_bold,
        song_info[1]
    );
    draw_text_mut(
        &mut img,
        Rgba([0, 0, 0, 255]),
        artistc.0,
        artistc.1 + 336,
        artistscale,
        &fonts.artist.regular,
        song_info[0]
    );
    draw_text_mut(
        &mut img,
        Rgba([0, 0, 0, 255]),
        scrobblesc.0,
        scrobblesc.1 + 529,
        scrobblescale,
        &fonts.number.extra_bold,
        &scrobbles.to_string()
    );
    Ok(img)
}