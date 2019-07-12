use embedded_graphics::icoord;
use embedded_graphics::image::ImageBmp;
use embedded_graphics::pixelcolor::FromRawData;
use embedded_graphics::pixelcolor::{Rgb565, Rgb888};
use embedded_graphics::prelude::*;
use embedded_graphics::Drawing;
use embedded_graphics::{egline, primitives::Line, style::Style};
use embedded_graphics::{fonts::Font8x16, prelude::*};
use marquee::{Flushable, Hub75, Marquee, Scrollable};
use rand::distributions::Uniform;
use rand::prelude::*;
use rosc::OscPacket;
use rpi_led_matrix::{LedCanvas, LedColor, LedFont, LedMatrix, LedMatrixOptions};
use std::env;
use std::f64::consts::PI;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::marker::PhantomData;
use std::net::{SocketAddrV4, UdpSocket};
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{thread, time};

fn main() {
    let mut screen = Hub75::new();
    let mut s = Scrollable::<Hub75, Rgb888>::new(screen);
    let mut marquee = Marquee::<Hub75, Rgb888>::new(s);

    loop {
        display_img(
            &mut marquee,
            "resources/megacorp.bmp",
            time::Duration::new(10, 0),
            true,
        );

        let msgs = vec![
            "¯\\_(ツ)_/¯",
            "AAAA  AAAA",
            "BBBB  BBBB",
            "THE WORLD DOES NOT OWE YOU A PLEASANT EXPERIENCE",
            "WE ARE NOT UNITED BY OUR DREAMS BUT BY OUR NIGHTMARES",
        ];

        let f = File::open("resources/default.txt").unwrap();
        let f = BufReader::new(f);

        let default_msgs: Vec<String> = f.lines().into_iter().map(|m| m.unwrap()).collect();
        display_msgs(&mut marquee, &default_msgs, 2);
        //display_msgs(&mut marquee, msgs, 2);
        display_msg(&mut marquee, "this is it", time::Duration::new(5, 0));
        display_msg(
            &mut marquee,
            "this is the sign you've been looking for",
            time::Duration::new(40, 0),
        );
    }
}

fn display_msgs<'a, I, J>(marquee: &mut Marquee<Hub75, Rgb888>, msgs: I, n: u32)
where
    I: IntoIterator<Item = &'a J>,
    J: AsRef<str> + 'a,
{
    let color = Rgb888::new(80, 0, 0);
    let images: Vec<Font8x16<Rgb888>> = msgs
        .into_iter()
        .map(AsRef::as_ref)
        .map(|msg| {
            Font8x16::render_str(msg)
                .translate(Coord::new(0, 1))
                .stroke_width(3)
                .stroke(Some(color))
        })
        .collect();
    marquee.scroll_n_times(images, n);
}

fn display_msg(marquee: &mut Marquee<Hub75, Rgb888>, msg: &str, duration: time::Duration) {
    let color = Rgb888::new(80, 0, 0);
    let image: Font8x16<Rgb888> = Font8x16::render_str(msg)
        .translate(Coord::new(0, 1))
        .stroke_width(3)
        .stroke(Some(color));
    marquee.scroll_for_duration(image, duration);
}

fn display_img(
    marquee: &mut Marquee<Hub75, Rgb888>,
    file: &str,
    duration: time::Duration,
    scroll: bool,
) {
    let mut f = File::open(file).unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    let mut img: ImageBmp<Rgb888> = ImageBmp::new(&buffer).unwrap();
    if scroll {
        marquee.scroll_for_duration(&img, duration);
    } else {
        //marquee.display_for_duration(img, duration);
    }
}
