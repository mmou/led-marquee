use rpi_led_matrix::{LedColor, LedFont, LedMatrix, LedMatrixOptions};
use std::path::Path;
use std::{thread, time};

const SCROLL_WAIT_NS: u32 = 50000000; // 0.5 sec

struct Marquee {
    matrix: LedMatrix,
    // perm:
    // temp:
}

struct Message {
    text: String,
    color: LedColor,
    duration: time::Duration,
}

impl Marquee {
    pub fn new() -> Self {
        let mut options = LedMatrixOptions::new();
        options.set_hardware_mapping("adafruit-hat");
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
        Marquee { matrix }
    }

    fn scroll(&mut self, msg: Message) {
        let mut canvas = self.matrix.canvas();

        let font = LedFont::new(Path::new("/home/pi/10x20.bdf")).unwrap();
        let (width, height) = canvas.size();
        let baseline = height - 2; //height / 2;

        canvas = self.matrix.offscreen_canvas();

        let now = time::Instant::now();
        let mut x = 0;
        while now.elapsed() < msg.duration {
            canvas.clear();
            canvas.draw_text(&font, &msg.text, x % width, baseline, &msg.color, 0, false);
            x = x + 1;
            canvas = self.matrix.swap(canvas);
            thread::sleep(time::Duration::new(0, SCROLL_WAIT_NS));
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
        duration: time::Duration::new(10, 0),
    };
    marquee.scroll(msg);
}
