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
use std::net::{SocketAddrV4, UdpSocket};
use std::path::Path;
use std::str::FromStr;
use std::{thread, time};

use std::f64::consts::PI;

const SCROLL_WAIT_NS: u32 = 50000000; // 0.5 sec

struct Marquee {
    matrix: LedMatrix,
    offscreen: LedCanvas,
    // perm:
    // temp:
}

struct Message {
    text: String,
    color: LedColor,
    duration: time::Duration,
}

impl Drawing<LedColor> for Marquee {
    fn draw<T>(&mut self, item: T)
    where
        T: IntoIterator<Item = Pixel<LedColor>>,
    {
        for Pixel(coord, color) in item {
            self.offscreen.set(coord[0] as i32, coord[1] as i32, &color);
        }
    }
}

impl Marquee {
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
        Marquee { matrix, offscreen }
    }

    pub fn flush(&mut self) {
        self.offscreen = self.matrix.swap(self.offscreen.clone());
        self.offscreen.clear();
    }

    fn scroll(&mut self, msg: Message) {
        let mut canvas = self.matrix.canvas();

        let font = LedFont::new(Path::new("/home/pi/10x20.bdf")).unwrap();
        let (width, height) = canvas.size();
        let baseline = height - 2; //height / 2;

        canvas = self.matrix.offscreen_canvas();

        let now = time::Instant::now();
        let text_width: i32 = 10 * ((msg.text.len() as i32) + 2);
        let mut x: i32 = 0;
        while now.elapsed() < msg.duration {
            canvas.clear();
            canvas.draw_text(&font, &msg.text, x % width, baseline, &msg.color, 0, false);
            canvas.draw_text(
                &font,
                &msg.text,
                x % width - text_width,
                baseline,
                &msg.color,
                0,
                false,
            );
            canvas.draw_text(
                &font,
                &msg.text,
                x % width + text_width,
                baseline,
                &msg.color,
                0,
                false,
            );
            x = x + 1;
            canvas = self.matrix.swap(canvas);
            thread::sleep(time::Duration::new(0, SCROLL_WAIT_NS));
        }
    }

    fn draw_line(&mut self) {
        let mut canvas = self.matrix.canvas();

        let (width, height) = canvas.size();

        canvas = self.matrix.offscreen_canvas();

        let mut color = LedColor {
            red: 127,
            green: 0,
            blue: 0,
        };

        canvas.clear();
        for x in 0..width {
            color.blue = 255 - 3 * x as u8;
            canvas.draw_line(x, 0, width - 1 - x, height - 1, &color);
            canvas = self.matrix.swap(canvas);
            thread::sleep(time::Duration::new(0, 10000000));
        }
    }

    fn draw_gradient(&mut self) {
        let mut canvas = self.matrix.canvas();
        let mut color = LedColor {
            red: 0,
            green: 0,
            blue: 0,
        };
        let period = 400;
        let duration = time::Duration::new(3, 0);
        let sleep_duration = duration / period;

        for t in 0..period {
            let t = t as f64;
            color.red = ((PI * t / period as f64).sin() * 255.) as u8;
            color.green = ((2. * PI * t / period as f64).cos() * 255.) as u8;
            color.blue = ((3. * PI * t / period as f64 + 0.3).cos() * 255.) as u8;
            canvas.set(1, 1, &color);
            canvas.set(0, 0, &color);
            thread::sleep(sleep_duration);
        }
    }

    fn draw_text_8x16(&mut self) {
        let styled_text: Font8x16<Rgb888> =
            text_8x16!("Hello world!", stroke = Some(Rgb888::new(100, 0, 0)));
        self.draw(styled_text.into_iter().map(|p| {
            let tmp: Rgb888 = p.1.into();
            let raw: u32 = tmp.into();
            Pixel(p.0, LedColor::from_raw_data(raw))
        }));
        self.flush();
        thread::sleep(time::Duration::new(3, 0));
    }

    fn draw_bmp(&mut self) {
        let image: ImageBmp<Rgb565> = ImageBmp::new(include_bytes!("../priceless.bmp")).unwrap();
        self.draw(image.into_iter().map(|p| {
            let tmp: Rgb888 = p.1.into();
            let raw: u32 = tmp.into();
            Pixel(p.0, LedColor::from_raw_data(raw))
        }));
        self.flush();
        thread::sleep(time::Duration::new(20, 0));
    }

    fn scroll_bmp(&mut self) {
        let (width, height) = self.offscreen.size();
        let mut image: ImageBmp<Rgb565> = ImageBmp::new(include_bytes!("../megacorp.bmp")).unwrap();

        let image_width: i32 = image.width() as i32;
        let duration = time::Duration::new(20, 0);
        let now = time::Instant::now();
        let mut x: i32 = 0;
        while now.elapsed() < duration {
            let temp_image = image.translate(icoord!(-x % image_width, 0));
            self.draw(temp_image.into_iter().map(|p| {
                let tmp: Rgb888 = p.1.into();
                let raw: u32 = tmp.into();
                Pixel(p.0, LedColor::from_raw_data(raw))
            }));
            let temp_image_right = temp_image.translate(icoord!(image_width, 0));
            self.draw(temp_image_right.into_iter().map(|p| {
                let tmp: Rgb888 = p.1.into();
                let raw: u32 = tmp.into();
                Pixel(p.0, LedColor::from_raw_data(raw))
            }));

            let temp_image_left = temp_image.translate(icoord!(-image_width, 0));
            self.draw(temp_image_left.into_iter().map(|p| {
                let tmp: Rgb888 = p.1.into();
                let raw: u32 = tmp.into();
                Pixel(p.0, LedColor::from_raw_data(raw))
            }));
            self.flush();
            x = x + 1;
            thread::sleep(time::Duration::new(0, SCROLL_WAIT_NS / 100));
        }
    }
}

fn main() {
    let mut marquee = Marquee::new();

    let color = LedColor {
        red: 0,
        green: 127,
        blue: 0,
    };
    let msg = Message {
        text: "IT'S WORKING".to_string(),
        color: color,
        duration: time::Duration::new(20, 0),
    };
    // marquee.draw_line();
    // marquee.scroll(msg);
    // marquee.draw_gradient();
    // marquee.draw_text_8x16();
    marquee.scroll_bmp();
}

fn main2() {
    let args: Vec<String> = env::args().collect();
    let usage = format!("Usage {} IP:PORT", &args[0]);
    if args.len() < 2 {
        println!("{}", usage);
        ::std::process::exit(1)
    }
    let addr = match SocketAddrV4::from_str(&args[1]) {
        Ok(addr) => addr,
        Err(_) => panic!(usage),
    };
    let sock = UdpSocket::bind(addr).unwrap();
    println!("Listening to {}", addr);

    let mut buf = [0u8; rosc::decoder::MTU];

    loop {
        match sock.recv_from(&mut buf) {
            Ok((size, addr)) => {
                println!("Received packet with size {} from: {}", size, addr);
                let packet = rosc::decoder::decode(&buf[..size]).unwrap();
                handle_packet(packet);
            }
            Err(e) => {
                println!("Error receiving from socket: {}", e);
                break;
            }
        }
    }
}

fn handle_packet(packet: OscPacket) {
    match packet {
        OscPacket::Message(msg) => {
            println!("OSC address: {}", msg.addr);
            match msg.args {
                Some(args) => {
                    println!("OSC arguments: {:?}", args);
                }
                None => println!("No arguments in message."),
            }
        }
        OscPacket::Bundle(bundle) => {
            println!("OSC Bundle: {:?}", bundle);
        }
    }
}
