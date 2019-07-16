use embedded_graphics::fonts::font_builder::FontBuilder;
use embedded_graphics::fonts::{Font8x16, Font8x16Conf};
use embedded_graphics::image::ImageBmp;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use marquee::{Hub75, Marquee, Scrollable};
use rand::seq::SliceRandom;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::time;

struct Text<'a> {
    text: &'a str,
    color: Rgb888,
}

impl<'a> Text<'a> {
    fn new(text: &'a str, color: Rgb888) -> Self {
        Text { text, color }
    }
}

fn create_texts<'a>(txts: Vec<Text<'a>>) -> Vec<FontBuilder<'a, Rgb888, Font8x16Conf>> {
    let images: Vec<Font8x16<Rgb888>> = txts
        .iter()
        .map(|t| {
            Font8x16::render_str(t.text)
                .translate(Coord::new(0, 1))
                .stroke_width(3)
                .stroke(Some(t.color))
        })
        .collect();
    images
}

fn create_text<'a>(txt: Text<'a>) -> FontBuilder<'a, Rgb888, Font8x16Conf> {
    let image: Font8x16<Rgb888> = Font8x16::render_str(&txt.text)
        .translate(Coord::new(0, 1))
        .stroke_width(3)
        .stroke(Some(txt.color));
    image
}

fn read_file<'a>(file: &'a str) -> Vec<u8> {
    let mut f = File::open(file).unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    buffer
}

fn create_image<'a>(data: &'a [u8]) -> ImageBmp<Rgb888> {
    let img: ImageBmp<Rgb888> = ImageBmp::new(data).unwrap();
    img
}

fn main() {
    let screen = Hub75::new();
    let s = Scrollable::<Hub75, Rgb888>::new(screen);
    let mut marquee = Marquee::<Hub75, Rgb888>::new(s);

    let f = File::open("resources/light_no_credit.txt").unwrap();
    let f = BufReader::new(f);
    let mut txts: Vec<String> = f.lines().into_iter().map(|l| l.unwrap()).collect();
    let mut rng = rand::thread_rng();
    txts.shuffle(&mut rng);

    let txts: Vec<Text> = txts
        .iter()
        .map(|l| Text::new(&l, Rgb888::new(20, 100, 130)))
        .collect();
    let imgs: Vec<Font8x16<Rgb888>> = create_texts(txts);

    let bytes: Vec<u8> = read_file("resources/megacorp.bmp");
    let img = create_image(&bytes);

    loop {
        marquee.scroll_n_times(&imgs, 1);
        marquee.scroll_for_duration(&img, time::Duration::new(10, 0));
    }
}
