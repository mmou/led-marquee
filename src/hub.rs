use embedded_graphics::drawable::Dimensions;
use embedded_graphics::fonts::Font8x16;
use embedded_graphics::icoord;
use embedded_graphics::image::ImageBmp;
use embedded_graphics::pixelcolor::{FromRawData, Rgb888};
use embedded_graphics::prelude::*;
use embedded_graphics::Drawing;
use graphics::ImageSize;
use rpi_led_matrix::{LedCanvas, LedColor, LedMatrix, LedMatrixOptions};
use std::marker::PhantomData;
use std::slice::Iter;
use std::str::FromStr;
use std::{thread, time};

const SCREEN_WIDTH: u32 = 32 * 4;
const SCROLL_WAIT_NS: u32 = 20 * 1000 * 1000;

pub trait Flushable {
    fn flush(&mut self);
}

/// Hub75 wraps LedMatrix
pub struct Hub75 {
    matrix: LedMatrix,
    offscreen: LedCanvas,
}

unsafe impl Send for Hub75 {}

impl Flushable for Hub75 {
    fn flush(&mut self) {
        self.offscreen = self.matrix.swap(self.offscreen.clone());
        self.offscreen.clear();
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
}

/// Scrollable is a wrapper around a Drawing with associated data about how to translate the
/// desired image
pub struct Scrollable<T, U>
where
    T: Drawing<U>,
    U: PixelColor,
{
    pub screen: T,
    max_x: u32,
    offset_x: i32,
    wrap: bool,
    pixel_type: PhantomData<U>,
}

impl<T, U> Drawing<U> for Scrollable<T, U>
where
    T: Drawing<U>,
    U: PixelColor,
{
    /// draws the image horizontally translated by self.offset_x
    fn draw<V>(&mut self, item: V)
    where
        V: IntoIterator<Item = Pixel<U>>,
    {
        let translated_item: Vec<Pixel<U>> = item
            .into_iter()
            .map(|Pixel(coord, color)| {
                let mut new_x: i32 = coord[0] as i32 + self.offset_x;
                if self.wrap {
                    new_x = (new_x + self.max_x as i32) % self.max_x as i32;
                }
                Pixel(UnsignedCoord::new(new_x as u32, coord[1]), color)
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
            max_x: SCREEN_WIDTH,
            offset_x: 0,
            wrap: true,
            pixel_type: PhantomData,
        }
    }

    pub fn inc_x(&mut self, x: i32) {
        if self.wrap {
            self.offset_x = (self.offset_x + x + self.max_x as i32) % self.max_x as i32;
        } else {
            self.offset_x = self.offset_x + x;
        }
    }

    pub fn set_x(&mut self, x: u32) {
        self.offset_x = x as i32;
    }

    pub fn set_wrap(&mut self, wrap: bool) {
        self.wrap = wrap;
    }

    pub fn set_width(&mut self, x: u32) {
        self.max_x = SCREEN_WIDTH;
        if x >= SCREEN_WIDTH {
            self.max_x = x;
        }
    }
}

/// Marquee contains a Scrollable
pub struct Marquee<T, U>
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

    /// scroll through the list of images n times
    pub fn scroll_n_times<'a, I, V>(&mut self, images: I, n: u32)
    where
        I: IntoIterator<Item = &'a V> + Clone,
        V: IntoIterator<Item = Pixel<U>> + Clone + Dimensions + 'a,
    {
        self.display.set_wrap(false);
        for _i in 0..n {
            for image in images.clone() {
                self.display.set_x(SCREEN_WIDTH);
                let width = image.size()[0];
                self.display.set_width(width);
                let mut prev = time::Instant::now();
                for _j in 0..(width + SCREEN_WIDTH) {
                    let now = time::Instant::now();
                    self.display.inc_x(-1);
                    self.display.draw(image.clone());
                    self.display.screen.flush();

                    match time::Duration::new(0, SCROLL_WAIT_NS)
                        .checked_sub(now.duration_since(prev))
                    {
                        Some(d) => thread::sleep(d),
                        None => (),
                    }
                    prev = now;
                }
            }
        }
    }

    /// scroll the given image for duration time
    pub fn scroll_for_duration<V>(&mut self, image: V, duration: time::Duration)
    where
        V: IntoIterator<Item = Pixel<U>> + Dimensions + Copy + Clone,
    {
        self.display.set_wrap(true);
        self.display.set_x(SCREEN_WIDTH);
        self.display.set_width(image.size()[0] + 32);
        let start = time::Instant::now();
        let mut prev = start;

        while start.elapsed() < duration {
            let now = time::Instant::now();
            self.display.inc_x(-1);
            self.display.draw(image.into_iter());
            self.display.screen.flush();

            match time::Duration::new(0, SCROLL_WAIT_NS).checked_sub(now.duration_since(prev)) {
                Some(d) => thread::sleep(d),
                None => (),
            }
            prev = now;
        }
    }

    /// display the given image for duration time
    pub fn display_for_duration<V>(&mut self, image: V, duration: time::Duration)
    where
        V: IntoIterator<Item = Pixel<U>> + Dimensions + Clone,
    {
        self.display.set_x(0);
        self.display.draw(image.clone());
        self.display.screen.flush();
        thread::sleep(duration);
    }
}
