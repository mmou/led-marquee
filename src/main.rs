use embedded_graphics::fonts::font_builder::{FontBuilder, FontBuilderConf};
use embedded_graphics::fonts::{Font8x16, Font8x16Conf};
use embedded_graphics::image::ImageBmp;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use marquee::{Hub75, Marquee, Scrollable};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::iter::FromIterator;
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
    let color = Rgb888::new(80, 0, 0);
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

    loop {
        // take in vector of strings
        let s: Vec<&str> = vec![
            "¯\\_(ツ)_/¯",
            "AAAA  AAAA",
            "BBBB  BBBB",
            "THE WORLD DOES NOT OWE YOU A PLEASANT EXPERIENCE",
            "WE ARE NOT UNITED BY OUR DREAMS BUT BY OUR NIGHTMARES",
        ];
        let txts: Vec<Text> = s
            .iter()
            .map(|m| Text::new(m, Rgb888::new(80, 0, 0)))
            .collect();

        // read in newline-delineated strings
        let f = File::open("resources/default.txt").unwrap();
        let f = BufReader::new(f);
        let default_txts: Vec<String> = f.lines().into_iter().map(|l| l.unwrap()).collect();

        let default_txts: Vec<Text> = default_txts
            .iter()
            .map(|l| Text::new(&l, Rgb888::new(0, 0, 80)))
            .collect();

        // scroll through iterator of strings 2 times
        let imgs: Vec<Font8x16<Rgb888>> = create_texts(default_txts);
        marquee.scroll_n_times(imgs, 2);
        let imgs: Vec<Font8x16<Rgb888>> = create_texts(txts);
        marquee.scroll_n_times(imgs, 2);

        // scroll short message for duration
        marquee.scroll_for_duration(
            create_text(Text::new("this is it", Rgb888::new(20, 0, 100))),
            time::Duration::new(5, 0),
        );

        // scroll long messsage for duration
        marquee.scroll_for_duration(
            create_text(Text::new(
                "this is the sign you've been looking for",
                Rgb888::new(10, 100, 100),
            )),
            time::Duration::new(40, 0),
        );

        let bytes: Vec<u8> = read_file("resources/megacorp.bmp");

        let img = create_image(&bytes);

        // display non-scrolling bmp
        marquee.display_for_duration(&img, time::Duration::new(10, 0));

        // display scrolling bmp
        marquee.scroll_for_duration(&img, time::Duration::new(10, 0));
    }
}
