use crate::calculations::GenreMonths;
use ab_glyph::{FontRef, PxScale};
use aho_corasick::AhoCorasick;
use chrono::{TimeZone, Utc};
use image::{
    imageops::{self, FilterType},
    DynamicImage, ImageReader, Rgba,
};
use imageproc::drawing::{draw_text_mut, text_size};
use itertools::Itertools;
use rand::seq::SliceRandom;
use regex::Regex;
use serde_json::{json, Value};
use std::error::Error;
use thousands::Separable;
use titlecase::titlecase;
use unicode_truncate::UnicodeTruncateStr;

#[derive(Clone)]
struct SpotifyFont<'a> {
    regular: FontRef<'a>,
    medium: FontRef<'a>,
    bold: FontRef<'a>,
    extra_bold: FontRef<'a>,
    narrow: FontRef<'a>,
}

impl<'b> SpotifyFont<'b> {
    pub fn new(fonts: Vec<FontRef<'b>>) -> Self {
        Self {
            regular: fonts[0].clone(),
            medium: fonts[1].clone(),
            bold: fonts[2].clone(),
            extra_bold: fonts[3].clone(),
            narrow: fonts[4].clone(),
        }
    }
}

struct SongFonts<'a> {
    artist: SpotifyFont<'a>,
    name: SpotifyFont<'a>,
    number: SpotifyFont<'a>,
}

impl<'b> SongFonts<'b> {
    pub fn new(font: SpotifyFont<'b>, fallback_font: SpotifyFont<'b>, strs: &[&str]) -> Self {
        Self {
            artist: if check_for_cyrillic(strs[0]) {
                fallback_font.clone()
            } else {
                font.clone()
            },
            name: if check_for_cyrillic(strs[1]) {
                fallback_font.clone()
            } else {
                font.clone()
            },
            number: font.clone(),
        }
    }
}

fn calculate_text_centre(
    img: &DynamicImage,
    scale: PxScale,
    font: &FontRef,
    text: &str,
) -> (i32, i32) {
    let text_size = text_size(scale, font, text);
    let image_size = (img.width(), img.height());
    let txtx: i32 = (image_size.0 / 2)
        .wrapping_sub(text_size.0 / 2)
        .try_into()
        .unwrap_or(0);
    let txty: i32 = (image_size.1 / 2)
        .wrapping_sub(text_size.1 / 2)
        .try_into()
        .unwrap_or(0);

    (txtx, txty)
}

fn batch_draw_text(
    strs: Vec<&str>,
    xys: Vec<(i32, i32)>,
    img: &mut DynamicImage,
    scale: PxScale,
    fonts: Vec<&FontRef>,
    offset: (i32, i32),
) {
    for i in 0..strs.len() {
        draw_text_mut(
            img,
            Rgba([255, 255, 255, 255]),
            xys[i].0 + offset.0,
            xys[i].1 + offset.1,
            scale,
            fonts[i],
            strs[i],
        );
    }
}

fn split_adv(s: &str) -> Vec<String> {
    let re = Regex::new(r"\w+|[^\w]").unwrap();
    let matches = re
        .find_iter(s)
        .map(|m| m.as_str().to_string())
        .collect::<Vec<String>>();
    matches
}

fn vec_substring(vec: &Vec<&String>, substr: &str) -> bool {
    let mut ret = false;
    for i in vec {
        if i.contains(substr) {
            ret = true;
            break;
        }
    }
    ret
}

fn trunc(s: &str, to: usize) -> String {
    if s.len() > to {
        s.unicode_truncate(to).0.trim_matches(' ').to_owned() + "..."
    } else {
        s.to_owned()
    }
}

fn fonts() -> Result<SpotifyFont<'static>, Box<dyn Error>> {
    let sf = SpotifyFont::new(vec![
        FontRef::try_from_slice(include_bytes!("../fonts/SpotifyMix-Regular.ttf"))?,
        FontRef::try_from_slice(include_bytes!("../fonts/SpotifyMix-Medium.ttf"))?,
        FontRef::try_from_slice(include_bytes!("../fonts/SpotifyMix-Bold.ttf"))?,
        FontRef::try_from_slice(include_bytes!("../fonts/SpotifyMix-Extrabold.ttf"))?,
        FontRef::try_from_slice(include_bytes!("../fonts/SpotifyMixNarrow-Black.ttf"))?,
    ]);
    Ok(sf)
}

fn fallback_fonts() -> Result<SpotifyFont<'static>, Box<dyn Error>> {
    let sf = SpotifyFont::new(vec![
        FontRef::try_from_slice(include_bytes!("../fonts/NotoSansJP-Light.otf"))?,
        FontRef::try_from_slice(include_bytes!("../fonts/NotoSansJP-Regular.otf"))?,
        FontRef::try_from_slice(include_bytes!("../fonts/NotoSansJP-Bold.otf"))?,
        FontRef::try_from_slice(include_bytes!("../fonts/NotoSansJP-Black.otf"))?,
        FontRef::try_from_slice(include_bytes!("../fonts/SpotifyMixNarrow-Black.ttf"))?,
    ]);
    Ok(sf)
}

// not cyrillic, it checks if the text isnt latin but im too lazy to rename
fn check_for_cyrillic(str: &str) -> bool {
    let latin_chars = (0x0020..=0x024F)
        .map(|x| char::from_u32(x).unwrap())
        .collect::<Vec<_>>();
    let punct = (0x2000..=0x206F)
        .map(|x| char::from_u32(x).unwrap())
        .collect::<Vec<_>>();
    let mut ret = false;
    for c in str.chars() {
        if !latin_chars.contains(&c) && !punct.contains(&c) {
            ret = true;
            break;
        }
    }
    ret
}

pub fn minutes_listened(
    total: i64,
    busiest_day: i64,
    busiest_time: i64,
) -> Result<DynamicImage, Box<dyn Error>> {
    let mut img = ImageReader::open("imgs/minuteslistened.png")?.decode()?;
    let fonts = fonts().unwrap();
    let totalscale = PxScale::from(290.0);
    let busiestscale = PxScale::from(50.0);
    let busiest_day_string = Utc
        .timestamp_opt(busiest_day, 0)
        .unwrap()
        .format("%B %_d")
        .to_string();
    let total_str = total.separate_with_commas();

    let totalc = calculate_text_centre(&img, totalscale, &fonts.extra_bold, &total_str);
    let busiestc = calculate_text_centre(
        &img,
        busiestscale,
        &fonts.medium,
        &format!(
            "Biggest listening day: {} with {} minutes",
            busiest_day_string, busiest_time
        ),
    );
    let busiest_day_len = text_size(busiestscale, &fonts.extra_bold, &busiest_day_string);
    let busiest_time_len = text_size(busiestscale, &fonts.extra_bold, &busiest_time.to_string());
    let busiest_str_len = (
        text_size(busiestscale, &fonts.medium, "Biggest listening day: "),
        text_size(busiestscale, &fonts.medium, " with "),
    );

    let busiest_day_coords = (busiestc.0 + (busiest_str_len.0 .0 as i32), busiestc.1);
    let busiest_with_coords = (
        busiest_day_coords.0 + (busiest_day_len.0 as i32),
        busiest_day_coords.1,
    );
    let busiest_time_coords = (
        busiest_with_coords.0 + (busiest_str_len.1 .0 as i32),
        busiest_with_coords.1,
    );
    let busiest_min_coords = (
        busiest_time_coords.0 + (busiest_time_len.0 as i32),
        busiest_time_coords.1,
    );

    // draw total minutes
    // in official image text is offset from bottom by 378 / 2 px
    draw_text_mut(
        &mut img,
        Rgba([255, 255, 255, 255]),
        totalc.0,
        totalc.1 - 189,
        totalscale,
        &fonts.extra_bold,
        &total_str,
    );
    batch_draw_text(
        vec![
            "Biggest listening day: ",
            &busiest_day_string,
            " with ",
            &busiest_time.to_string(),
            " minutes",
        ],
        vec![
            busiestc,
            busiest_day_coords,
            busiest_with_coords,
            busiest_time_coords,
            busiest_min_coords,
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
        (0, 132),
    );

    Ok(img)
}

pub fn top_song(
    name: String,
    scrobbles: i64,
    cover: DynamicImage,
) -> Result<DynamicImage, Box<dyn Error>> {
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
    let trackc = calculate_text_centre(&img, songscale, &fonts.name.extra_bold, song_info[1]);
    // artist name
    let artistc = calculate_text_centre(&img, artistscale, &fonts.artist.regular, song_info[0]);
    // scrobbles
    let scrobblesc = calculate_text_centre(
        &img,
        scrobblescale,
        &fonts.number.regular,
        &scrobbles.to_string(),
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
        song_info[1],
    );
    draw_text_mut(
        &mut img,
        Rgba([0, 0, 0, 255]),
        artistc.0,
        artistc.1 + 336,
        artistscale,
        &fonts.artist.regular,
        song_info[0],
    );
    draw_text_mut(
        &mut img,
        Rgba([0, 0, 0, 255]),
        scrobblesc.0,
        scrobblesc.1 + 529,
        scrobblescale,
        &fonts.number.extra_bold,
        &scrobbles.to_string(),
    );
    Ok(img)
}

pub fn top_5_songs(
    songs: Vec<(&String, (DynamicImage, &i32))>,
) -> Result<DynamicImage, Box<dyn Error>> {
    let mut img = ImageReader::open("imgs/top5songs.png")?.decode()?;
    let scale = PxScale::from(48.0);
    let mut titlexy = (412, 585);
    let mut artistxy = (412, 643);
    let mut coverxy = (172, 534);
    let coverscale = (208, 208);
    for song in songs {
        let song_info = song.0.split(" - ").collect::<Vec<_>>();
        let fonts = SongFonts::new(fonts()?, fallback_fonts()?, &song_info);
        let scaled_cover = song
            .1
             .0
            .resize(coverscale.0, coverscale.1, FilterType::CatmullRom);
        imageops::overlay(&mut img, &scaled_cover, coverxy.0, coverxy.1);
        draw_text_mut(
            &mut img,
            Rgba([0, 0, 0, 255]),
            titlexy.0,
            titlexy.1,
            scale,
            &fonts.name.bold,
            song_info[1],
        );
        draw_text_mut(
            &mut img,
            Rgba([0, 0, 0, 255]),
            artistxy.0,
            artistxy.1,
            scale,
            &fonts.artist.regular,
            song_info[0],
        );
        titlexy.1 += 240;
        artistxy.1 += 240;
        coverxy.1 += 240;
    }
    Ok(img)
}

pub fn genre_evolution(months: GenreMonths) -> Result<Vec<DynamicImage>, Box<dyn Error>> {
    let imgs = [
        ImageReader::open("imgs/genreevolution1.png")?.decode()?,
        ImageReader::open("imgs/genreevolution2.png")?.decode()?,
        ImageReader::open("imgs/genreevolution3.png")?.decode()?,
    ];
    let mut modified_imgs = Vec::with_capacity(3);
    let mut rng = rand::thread_rng();
    let mut existing_genres = Vec::new();

    // will probably be more patterns in the future
    let bad_patterns = &["russian", "belarusian"];
    let ac = AhoCorasick::new(bad_patterns)?;

    let font = fonts()?;
    let genrescale = PxScale::from(288.0);
    let monthscale = PxScale::from(62.0);
    let artistsscale = PxScale::from(48.0);
    let fallbackscale = PxScale::from(48.0);

    #[allow(clippy::needless_range_loop)]
    for i in 0..=2 {
        let mut img = imgs[i].clone();
        let genrehashmap = months.clone().get(i);
        let artists = genrehashmap.keys().collect::<Vec<&String>>();
        let top_artists = format!(
            "Listening to artists like {}, {}, and {}",
            artists[0], artists[1], artists[2]
        );
        let top_artists_wrapped = textwrap::wrap(&top_artists, 54);

        let mut genres = genrehashmap
            .values()
            .collect::<Vec<&Vec<Value>>>()
            .iter()
            .map(|x| x.choose(&mut rng).unwrap_or(&json!("")).to_string())
            .map(|x| {
                ac.replace_all(&x, &["", ""])
                    .trim_matches(&['\"', ' '])
                    .to_string()
            })
            .map(|x| titlecase(&x))
            .filter(|x| !existing_genres.contains(x))
            .filter(|x| !x.is_empty())
            .collect::<Vec<String>>()
            .choose_multiple(&mut rng, 2)
            .cloned()
            .collect::<Vec<String>>();
        if genres.len() < 2 {
            genres.push(existing_genres.choose(&mut rng).unwrap().to_string());
        }
        existing_genres.append(&mut genres.clone());
        existing_genres = existing_genres
            .iter()
            .unique()
            .cloned()
            .collect::<Vec<String>>();

        let monthc = calculate_text_centre(
            &img,
            monthscale,
            &font.medium,
            &format!("My {}", months.clone().get_month_string(i)),
        );
        let genres_wrapped = [
            textwrap::wrap(&genres[0], 15),
            textwrap::wrap(&genres[1], 15),
        ]
        .concat();

        draw_text_mut(
            &mut img,
            Rgba([255, 255, 255, 255]),
            monthc.0,
            519,
            monthscale,
            &font.medium,
            &format!("My {}", months.clone().get_month_string(i)),
        );
        let mut genrelinexy = (0, 593);
        for i in &genres_wrapped {
            genrelinexy.0 = calculate_text_centre(&img, genrescale, &font.narrow, i).0;
            draw_text_mut(
                &mut img,
                Rgba([255, 255, 255, 255]),
                genrelinexy.0,
                genrelinexy.1,
                genrescale,
                &font.narrow,
                i,
            );
            genrelinexy.1 += 250;
        }

        let mut artistlinexy = (0, genrelinexy.1 + 50);
        for i in &top_artists_wrapped {
            let words = split_adv(i);
            artistlinexy.0 =
                calculate_text_centre(&img, fallbackscale, &fallback_fonts()?.medium, i).0;
            for j in words {
                let f = if vec_substring(&artists, &j) {
                    if check_for_cyrillic(&j) {
                        fallback_fonts()?.bold
                    } else {
                        fonts()?.bold
                    }
                } else {
                    fonts()?.medium
                };
                let s = if check_for_cyrillic(&j) {
                    fallbackscale
                } else {
                    artistsscale
                };
                draw_text_mut(
                    &mut img,
                    Rgba([255, 255, 255, 255]),
                    artistlinexy.0,
                    artistlinexy.1,
                    s,
                    &f,
                    &j,
                );
                artistlinexy.0 += text_size(s, &f, &j).0 as i32;
            }
            artistlinexy.1 += 50;
        }
        modified_imgs.push(img);
    }

    Ok(modified_imgs)
}

pub fn final_image(
    total: i64,
    songs: Vec<&str>,
    artists: Vec<&str>,
    cover: DynamicImage,
) -> Result<DynamicImage, Box<dyn Error>> {
    let mut img = ImageReader::open("imgs/final.png")?.decode()?;
    let fallbackscale = PxScale::from(64.0);
    let scale = PxScale::from(64.0);
    let totalscale = PxScale::from(130.0);
    let total_str = total.separate_with_commas();

    let coverscale = (616, 616);
    let coverxy = (232, 192);
    let scaled_cover = cover.resize(coverscale.0, coverscale.1, FilterType::CatmullRom);
    imageops::overlay(&mut img, &scaled_cover, coverxy.0, coverxy.1);

    let mut artistxy = (124, 1102);
    let mut titlexy = (606, 1102);
    let totalxy = (80, 1516);
    for (a, t) in artists.iter().zip(songs.iter()) {
        let font = SongFonts::new(fonts()?, fallback_fonts()?, &[a, t]);
        draw_text_mut(
            &mut img,
            Rgba([255, 255, 255, 255]),
            artistxy.0,
            artistxy.1,
            if check_for_cyrillic(a) {
                fallbackscale
            } else {
                scale
            },
            &font.artist.bold,
            &trunc(a, 11),
        );
        draw_text_mut(
            &mut img,
            Rgba([255, 255, 255, 255]),
            titlexy.0,
            titlexy.1,
            if check_for_cyrillic(t) {
                fallbackscale
            } else {
                scale
            },
            &font.name.bold,
            &trunc(t, 14),
        );
        artistxy.1 += 56;
        titlexy.1 += 56;
    }
    draw_text_mut(
        &mut img,
        Rgba([255, 255, 255, 255]),
        totalxy.0,
        totalxy.1,
        totalscale,
        &fonts()?.extra_bold,
        &total_str,
    );

    Ok(img)
}
