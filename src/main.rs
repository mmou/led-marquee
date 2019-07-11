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
use std::io::{self, Write};
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
        // megaglitch(&ss, time::Duration::new(30, 0));

        text(
            &mut marquee,
            "THE WORLD DOES NOT OWE YOU A PLEASANT EXPERIENCE     WE ARE NOT UNITED BY OUR DREAMS BUT BY OUR NIGHTMARES     ".to_string(),
            time::Duration::new(200, 0),
        );

        // display_clue(&mut s2, time::Duration::new(24, 0));
    }
}

fn text(marquee: &mut Marquee<Hub75, Rgb888>, text: String, duration: time::Duration) {
    let color = Rgb888::new(80, 0, 0);
    let textt: Font8x16<Rgb888> = Font8x16::render_str(&text)
        .translate(Coord::new(0, 1))
        .stroke_width(3)
        .stroke(Some(color));
    //screen.draw_text_8x16(msg);
    io::stdout().flush().unwrap();
    marquee.scroll(textt, text.len() as u32 * 8, duration);
}

/*
fn display_clue(s: &mut Scrollable<Hub75, Rgb888>, duration: time::Duration) {
    let mut marquee = Marquee::<Hub75, Rgb888>::new(*s);

    let mut clue: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../priceless2.bmp")).unwrap();
    marquee.scroll(&clue, clue.width() + 6, duration);
}*/

/*
fn megaglitch(s: &Arc<Mutex<Scrollable<Hub75, Rgb888>>>, duration: time::Duration) {
    let mut image: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp.bmp")).unwrap();
    let mut image1: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp-1.bmp")).unwrap();
    let mut image2: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp-2.bmp")).unwrap();
    let mut image3: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp-3.bmp")).unwrap();
    let mut image4: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp-4.bmp")).unwrap();
    let mut image5: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../megacorp-5.bmp")).unwrap();

    let mut images = Arc::new(vec![image, image1, image2, image3, image4, image5]);
    let mut indices: Vec<usize> = vec![1, 2, 3, 4, 5];

    let mut rng = rand::thread_rng();

    let now = time::Instant::now();

    let clean_range = Uniform::new(100, 2000);
    let glitch_range = Uniform::new(10, 150);

    //let s = Arc::new(Mutex::new(s));
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

                s.set_image_width(images[0].width());
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
*/
