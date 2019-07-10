use embedded_graphics::icoord;
use embedded_graphics::image::ImageBmp;
use embedded_graphics::pixelcolor::FromRawData;
use embedded_graphics::pixelcolor::{Rgb565, Rgb888};
use embedded_graphics::prelude::*;
use embedded_graphics::Drawing;
use embedded_graphics::{egline, primitives::Line, style::Style};
use embedded_graphics::{fonts::Font8x16, prelude::*, text_8x16};
use rand::distributions::Uniform;
use rand::prelude::*;
use rosc::OscPacket;
use rpi_led_matrix::{LedCanvas, LedColor, LedFont, LedMatrix, LedMatrixOptions};
use std::env;
use std::f64::consts::PI;
use std::marker::PhantomData;
use std::net::{SocketAddrV4, UdpSocket};
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{thread, time};

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

unsafe impl Send for Hub75 {}

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
            //thread::sleep(time::Duration::new(0, SCROLL_WAIT_NS * 10));
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
        self.offset_x = (self.offset_x + x) % self.max_x as i32;
    }

    pub fn set_x(&mut self, x: i32) {
        self.offset_x = x;
    }

    pub fn set_image_width(&mut self, x: u32) {
        self.max_x = 32 * 4;
        if x > 32 * 4 {
            self.max_x = x;
        }
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
        options.set_brightness(40);
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
            thread::sleep(time::Duration::new(0, SCROLL_WAIT_NS));
        }
    }
}

fn main() {
    loop {
        megaglitch(time::Duration::new(30, 0));

        text(" A MESSAGE FROM  ".to_string(), time::Duration::new(2, 0));
        display_market(time::Duration::new(9, 0));

        for i in 1..4 {
            text("      BUY NOW  ".to_string(), time::Duration::new(1, 0));
            text("".to_string(), time::Duration::new(0, 500000000));
        }

        display_clue(time::Duration::new(24, 0));
    }
}

fn text(text: String, duration: time::Duration) {
    let mut screen = Hub75::new();

    let color = Rgb888::new(0, 100, 0);
    let msg = DisplayMessage {
        text,
        color,
        duration,
    };
    screen.draw_text_8x16(msg);
}

fn display_clue(duration: time::Duration) {
    let mut screen = Hub75::new();
    let mut s = Scrollable::<Hub75, Rgb888>::new(screen);
    let mut marquee = Marquee::<Hub75, Rgb888>::new(s);

    let mut clue: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../clue.bmp")).unwrap();
    marquee.scroll(&clue, clue.width() + 6, duration);
}

fn display_market(duration: time::Duration) {
    let mut screen = Hub75::new();
    let mut s = Scrollable::<Hub75, Rgb888>::new(screen);
    let mut marquee = Marquee::<Hub75, Rgb888>::new(s);

    let mut market: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../market.bmp")).unwrap();
    marquee.scroll(&market, market.width() + 12, duration);
}

fn clue_old(duration: time::Duration) {
    let mut screen = Hub75::new();
    let mut clue1: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../clue1.bmp")).unwrap();
    let mut clue2: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../clue2.bmp")).unwrap();
    let mut clue3: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../clue3.bmp")).unwrap();
    let mut clue4: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../clue4.bmp")).unwrap();
    let mut s = Scrollable::<Hub75, Rgb888>::new(screen);

    let mut marquee = Marquee::<Hub75, Rgb888>::new(s);

    for i in 1..3 {
        marquee.scroll(&clue1, clue1.width(), duration);
        marquee.scroll(&clue2, clue2.width(), duration);
        marquee.scroll(&clue3, clue3.width(), duration);
        marquee.scroll(&clue4, clue4.width(), duration);
    }
}

fn megaglitch(duration: time::Duration) {
    let mut screen = Hub75::new();

    let mut image: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp.bmp")).unwrap();
    let mut image1: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp-1.bmp")).unwrap();
    let mut image2: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp-2.bmp")).unwrap();
    let mut image3: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp-3.bmp")).unwrap();
    let mut image4: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp-4.bmp")).unwrap();
    let mut image5: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp-5.bmp")).unwrap();

    let mut images = Arc::new(vec![image, image1, image2, image3, image4, image5]);
    let mut indices: Vec<usize> = vec![1, 2, 3, 4, 5];

    let mut s = Scrollable::<Hub75, Rgb888>::new(screen);
    s.set_image_width(images[0].width());

    let mut rng = rand::thread_rng();

    let now = time::Instant::now();

    let clean_range = Uniform::new(100, 2000);
    let glitch_range = Uniform::new(10, 150);

    let s = Arc::new(Mutex::new(s));
    let i = Arc::new(Mutex::new(0 as usize));

    let cloned_s = s.clone();
    let cloned_images = images.clone();
    let cloned_i = i.clone();
    thread::spawn(move || {
        let s = cloned_s;
        let i = cloned_i;
        let images = cloned_images;
        while now.elapsed() < duration {
            {
                let mut s = s.lock().unwrap();
                let mut j = i.lock().unwrap();;
                s.inc_x(-1);
                s.draw(&images[*j]);
                s.screen.flush();
            }
            thread::sleep(Duration::from_millis(8));
        }
    });

    while now.elapsed() < duration {
        {
            let mut s = s.lock().unwrap();
            let mut j = i.lock().unwrap();
            *j = 0;
            s.draw(&images[0]);
            s.screen.flush();
        }
        thread::sleep(Duration::from_millis(rng.sample(clean_range)));

        indices.shuffle(&mut rng);

        for k in indices.iter() {
            {
                let mut s = s.lock().unwrap();
                let mut j = i.lock().unwrap();
                *j = *k;

                s.draw(&images[*k]);
                s.screen.flush();
            }
            thread::sleep(Duration::from_millis(rng.sample(glitch_range)));
        }
    }
}

fn infinimegaglitch() {
    let mut screen = Hub75::new();

    let mut image: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp.bmp")).unwrap();
    let mut image1: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp-1.bmp")).unwrap();
    let mut image2: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp-2.bmp")).unwrap();
    let mut image3: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp-3.bmp")).unwrap();
    let mut image4: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp-4.bmp")).unwrap();
    let mut image5: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp-5.bmp")).unwrap();

    let mut images = Arc::new(vec![image, image1, image2, image3, image4, image5]);
    let mut indices: Vec<usize> = vec![1, 2, 3, 4, 5];

    let mut s = Scrollable::<Hub75, Rgb888>::new(screen);
    s.set_image_width(images[0].width());

    let mut rng = rand::thread_rng();

    let clean_range = Uniform::new(100, 2000);
    let glitch_range = Uniform::new(10, 150);

    let s = Arc::new(Mutex::new(s));
    let i = Arc::new(Mutex::new(0 as usize));

    let cloned_s = s.clone();
    let cloned_images = images.clone();
    let cloned_i = i.clone();
    thread::spawn(move || {
        let s = cloned_s;
        let i = cloned_i;
        let images = cloned_images;
        loop {
            {
                let mut s = s.lock().unwrap();
                let mut j = i.lock().unwrap();;
                s.inc_x(-1);
                s.draw(&images[*j]);
                s.screen.flush();
            }
            thread::sleep(Duration::from_millis(8));
        }
    });

    loop {
        {
            let mut s = s.lock().unwrap();
            let mut j = i.lock().unwrap();
            *j = 0;
            s.draw(&images[0]);
            s.screen.flush();
        }
        thread::sleep(Duration::from_millis(rng.sample(clean_range)));

        indices.shuffle(&mut rng);

        for k in indices.iter() {
            {
                let mut s = s.lock().unwrap();
                let mut j = i.lock().unwrap();
                *j = *k;

                s.draw(&images[*k]);
                s.screen.flush();
            }
            thread::sleep(Duration::from_millis(rng.sample(glitch_range)));
        }
    }

    //screen.draw_bmp(&image, time::Duration::new(2, 0));
    //screen.scroll_bmp(&image, time::Duration::new(10, 0));
}
