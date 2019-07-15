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

fn main() {
    let screen = Hub75::new();
    let s = Scrollable::<Hub75, Rgb888>::new(screen);
    let mut marquee = Marquee::<Hub75, Rgb888>::new(s);

    loop {
        // take in vector of strings
        let msgs: Vec<&str> = vec![
            "¯\\_(ツ)_/¯",
            "AAAA  AAAA",
            "BBBB  BBBB",
            "THE WORLD DOES NOT OWE YOU A PLEASANT EXPERIENCE",
            "WE ARE NOT UNITED BY OUR DREAMS BUT BY OUR NIGHTMARES",
        ];

        // read in newline-delineated strings
        let f = File::open("resources/default.txt").unwrap();
        let f = BufReader::new(f);
        let default_msgs: Vec<String> = f.lines().into_iter().map(|m| m.unwrap()).collect();

        // scroll through iterator of strings 2 times
        let imgs: Vec<Font8x16<Rgb888>> = create_texts(&default_msgs);
        marquee.scroll_n_times(&imgs, 2);
        let imgs: Vec<Font8x16<Rgb888>> = create_texts(&msgs);
        marquee.scroll_n_times(&imgs, 2);

        // scroll short message for duration
        marquee.scroll_for_duration(create_text("this is it"), time::Duration::new(5, 0));

        // scroll long messsage for duration
        marquee.scroll_for_duration(
            create_text("this is the sign you've been looking for"),
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

fn create_texts<'a, I, S>(msgs: I) -> Vec<FontBuilder<'a, Rgb888, Font8x16Conf>>
where
    I: IntoIterator<Item = &'a S>,
    S: AsRef<str> + 'a,
{
    let color = Rgb888::new(80, 0, 0);
    let images: Vec<Font8x16<Rgb888>> = msgs
        .into_iter()
        .map(AsRef::as_ref)
        .map(|m| {
            Font8x16::render_str(m)
                .translate(Coord::new(0, 1))
                .stroke_width(3)
                .stroke(Some(color))
        })
        .collect();
    images
}

fn create_text<'a>(msg: &'a str) -> FontBuilder<'a, Rgb888, Font8x16Conf> {
    let color = Rgb888::new(80, 0, 0);
    let image: Font8x16<Rgb888> = Font8x16::render_str(msg)
        .translate(Coord::new(0, 1))
        .stroke_width(3)
        .stroke(Some(color));
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
