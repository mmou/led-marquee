use embedded_graphics::icoord;
use embedded_graphics::image::ImageBmp;
use embedded_graphics::pixelcolor::FromRawData;
use embedded_graphics::pixelcolor::{Rgb565, Rgb888};
use embedded_graphics::prelude::*;
use embedded_graphics::Drawing;
use embedded_graphics::{egline, primitives::Line, style::Style};
use embedded_graphics::{fonts::Font8x16, prelude::*, text_8x16};
use rosc::OscPacket;
use rpi_led_matrix::{LedCanvas, LedColor, LedFont, LedMatrix, LedMatrixOptions};
use std::env;
use std::marker::PhantomData;
use std::net::{SocketAddrV4, UdpSocket};
use std::path::Path;
use std::str::FromStr;
use std::{thread, time};

use std::f64::consts::PI;

const SCROLL_WAIT_NS: u32 = 5000000; // 0.05 sec

struct DisplayMessage {
    text: String,
    color: Rgb888,
    duration: time::Duration,
}

struct Hub75 {
    matrix: LedMatrix,
    offscreen: LedCanvas,
}

pub trait Flushable {
    fn flush(&mut self);
}

impl Flushable for Hub75 {
    fn flush(&mut self) {
        self.offscreen = self.matrix.swap(self.offscreen.clone());
        self.offscreen.clear();
    }
}

struct Marquee<T, U>
where
    T: Drawing<U> + Flushable,
    U: PixelColor,
{
    display: Scrollable<T, U>,
}

impl<T, U> Marquee<T, U>
where
    T: Drawing<U> + Flushable,
    U: PixelColor,
{
    pub fn new(display: Scrollable<T, U>) -> Self {
        Marquee { display }
    }

    pub fn scroll<V>(&mut self, image: V, width: u32, duration: time::Duration)
    where
        V: IntoIterator<Item = Pixel<U>> + Clone,
    {
        self.display.set_image_width(width);
        let now = time::Instant::now();
        let mut x: i32 = 0;
        while now.elapsed() < duration {
            self.display.set_x(-x);
            self.display.draw(image.clone());
            self.display.screen.flush();
            x = x + 1;
            thread::sleep(time::Duration::new(0, SCROLL_WAIT_NS * 10));
        }
    }
}

struct Scrollable<T, U>
where
    T: Drawing<U>,
    U: PixelColor,
{
    screen: T,
    max_x: u32,
    offset_x: i32,
    pixel_type: PhantomData<U>,
}

impl<T, U> Drawing<U> for Scrollable<T, U>
where
    T: Drawing<U>,
    U: PixelColor,
{
    fn draw<V>(&mut self, item: V)
    where
        V: IntoIterator<Item = Pixel<U>>,
    {
        let translated_item: Vec<Pixel<U>> = item
            .into_iter()
            .map(|Pixel(coord, color)| {
                Pixel(
                    UnsignedCoord::new(
                        ((coord[0] as i32 + self.offset_x + self.max_x as i32) % self.max_x as i32)
                            as u32,
                        coord[1],
                    ),
                    color,
                )
            })
            .collect();
        self.screen.draw(translated_item);
    }
}

impl<T, U> Scrollable<T, U>
where
    T: Drawing<U>,
    U: PixelColor,
{
    pub fn new(screen: T) -> Self {
        Scrollable {
            screen: screen,
            max_x: 0,
            offset_x: 0,
            pixel_type: PhantomData,
        }
    }

    pub fn inc_x(&mut self, x: i32) {
        self.offset_x = self.offset_x + x;
    }

    pub fn set_x(&mut self, x: i32) {
        self.offset_x = x;
    }

    pub fn set_image_width(&mut self, x: u32) {
        self.max_x = x;
    }
}

impl Drawing<LedColor> for Hub75 {
    fn draw<T>(&mut self, item: T)
    where
        T: IntoIterator<Item = Pixel<LedColor>>,
    {
        for Pixel(coord, color) in item {
            self.offscreen.set(coord[0] as i32, coord[1] as i32, &color);
        }
    }
}

impl Drawing<Rgb888> for Hub75 {
    fn draw<T>(&mut self, item: T)
    where
        T: IntoIterator<Item = Pixel<Rgb888>>,
    {
        for Pixel(coord, color) in item {
            self.offscreen.set(
                coord[0] as i32,
                coord[1] as i32,
                &LedColor::from_raw_data(color.into()),
            );
        }
    }
}

impl Hub75 {
    pub fn new() -> Self {
        let mut options = LedMatrixOptions::new();
        options.set_hardware_mapping("adafruit-hat-pwm");
        options.set_chain_length(4);
        options.set_hardware_pulsing(false);
        options.set_rows(16);
        options.set_cols(32);
        options.set_multiplexing(3);
        options.set_row_address_type(2);
        options.set_brightness(70);
        //options.set_pwm_lsb_nanoseconds(130);
        //options.set_inverse_colors(true);
        //options.set_refresh_rate(true);
        let matrix = LedMatrix::new(Some(options)).unwrap();
        let mut offscreen = matrix.offscreen_canvas();
        offscreen.clear();
        Hub75 { matrix, offscreen }
    }

    fn draw_text_8x16(&mut self, msg: DisplayMessage) {
        let styled_text: Font8x16<Rgb888> = Font8x16::render_str(&msg.text).stroke(Some(msg.color));
        self.draw(styled_text);
        self.flush();
        thread::sleep(msg.duration);
    }

    fn scroll_text_8x16(&mut self, msg: DisplayMessage) {
        let text: Font8x16<Rgb888> = Font8x16::render_str(&msg.text).stroke(Some(msg.color));

        let text_width: i32 = msg.text.len() as i32 * 8;
        let now = time::Instant::now();
        let mut x: i32 = 0;
        while now.elapsed() < msg.duration {
            let temp_text = text.translate(icoord!(-x % text_width, 0));
            self.draw(&temp_text);
            let temp_text_right = temp_text.translate(icoord!(text_width, 0));
            self.draw(&temp_text_right);

            let temp_text_left = temp_text.translate(icoord!(-text_width, 0));
            self.draw(&temp_text_left);
            self.flush();
            x = x + 1;
            thread::sleep(time::Duration::new(0, SCROLL_WAIT_NS * 10));
        }
    }

    fn draw_bmp(&mut self, image: &ImageBmp<Rgb888>, duration: time::Duration) {
        self.draw(image);
        self.flush();
        thread::sleep(duration);
    }

    fn scroll_bmp(&mut self, image: &ImageBmp<Rgb888>, duration: time::Duration) {
        let (width, height) = self.offscreen.size();

        let image_width: i32 = image.width() as i32;
        let now = time::Instant::now();
        let mut x: i32 = 0;
        while now.elapsed() < duration {
            let temp_image = image.translate(icoord!(-x % image_width, 0));
            self.draw(&temp_image);
            let temp_image_right = temp_image.translate(icoord!(image_width, 0));
            self.draw(&temp_image_right);

            let temp_image_left = temp_image.translate(icoord!(-image_width, 0));
            self.draw(&temp_image_left);
            self.flush();
            x = x + 1;
            thread::sleep(time::Duration::new(0, SCROLL_WAIT_NS * 10));
        }
    }
}

fn main() {
    let mut screen = Hub75::new();

    let color = Rgb888::new(100, 0, 0);
    let msg = DisplayMessage {
        text: "IT'S WORKING".to_string(),
        color: color,
        duration: time::Duration::new(5, 0),
    };
    //screen.scroll_text_8x16(msg);

    let mut image1: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp.bmp")).unwrap();
    let mut image2: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../priceless2.bmp")).unwrap();

    let mut s = Scrollable::<Hub75, Rgb888>::new(screen);

    let mut marquee = Marquee::<Hub75, Rgb888>::new(s);

    marquee.scroll(&image1, image1.width(), time::Duration::new(5, 0));
    marquee.scroll(&image2, image2.width(), time::Duration::new(5, 0));

    //screen.draw_bmp(&image, time::Duration::new(2, 0));
    //screen.scroll_bmp(&image, time::Duration::new(10, 0));
}
