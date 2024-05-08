/*
⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿
⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿⣿⣿ ⣿
⣿⡇ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⢸⣿ ⣿
⣿⣧  ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⠨⣿⣿ ⣿
⣿⣿⠨  ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⢸⣿ ⣿ ⣿
⣿ ⣿ ⣿         ⣿    ⣿ ⣿ ⣿ ⣿ ⣿  ⣿
*/

use ratatui::{
    buffer::Buffer,
    layout::*,
    style::{Color, Style},
    text::Line,
    widgets::Widget,
};
use std::fmt::Display;

#[derive(Copy, Clone)]
enum Key {
    Single,
    Backspace,
    Tab,
    Backslash,
    Caps,
    Enter,
    ShiftL,
    ShiftR,
    Space,
}

impl From<&Key> for &'static str {
    fn from(val: &Key) -> Self {
        match val {
            Key::Single => "⣿ ",
            Key::Backspace => "⣿⣿⣿ ",
            Key::Tab => "⣿⡇ ",
            Key::Backslash => "⢸⣿ ",
            Key::Caps => "⣿⣧  ",
            Key::Enter => "⣿⣿⣿ ",
            Key::ShiftL => "⣿⣿⠨  ",
            Key::ShiftR => "⢸⣿ ",
            Key::Space => "        ⣿     ",
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str: &'static str = self.into();
        write!(f, "{}", str)
    }
}

type Row = Vec<Key>;

pub struct Layout {
    rows: [Row; 6],
    pub colors: [ratatui::style::Color; 84],
}

impl Default for Layout {
    fn default() -> Self {
        let row_1 = vec![Key::Single; 16];
        let mut row_2 = vec![Key::Single; 13];
        row_2.extend([Key::Backspace, Key::Single]);
        let mut row_3 = vec![Key::Tab; 1];
        row_3.extend([Key::Single; 12]);
        row_3.extend([Key::Backslash, Key::Single]);
        let mut row_4 = vec![Key::Caps; 1];
        row_4.extend([Key::Single; 11]);
        row_4.extend([Key::Enter, Key::Single]);
        let mut row_5 = vec![Key::ShiftL; 1];
        row_5.extend([Key::Single; 10]);
        row_5.extend([Key::ShiftR, Key::Single, Key::Single]);
        let mut row_6 = vec![Key::Single; 3];
        row_6.extend([Key::Space; 1]);
        row_6.extend([Key::Single; 6]);

        Self {
            rows: [row_1, row_2, row_3, row_4, row_5, row_6],
            colors: [ratatui::style::Color::Black; 84],
        }
    }
}

impl Layout {
    pub fn print(&self) -> std::io::Result<()> {
        for row in &self.rows {
            for key in row {
                print!("{} ", key);
            }
            println!();
        }
        Ok(())
    }
}

pub struct LayoutWidget<'a> {
    pub layout: &'a Layout,
}

impl<'a> Widget for LayoutWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let mut x = 0;
        let mut absolute = 0;
        for (y, row) in self.layout.rows.iter().enumerate() {
            for key in row {
                let key_str: &'static str = key.into();
                let color = self.layout.colors[absolute];
                let line = Line::from(key_str).style(Style::default().fg(color));
                buf.set_line(area.x + x, area.y + y as u16, &line, line.width() as u16);
                x += line.width() as u16;
                absolute += 1;
            }
            x = 0;
        }
    }
}

pub struct VUMeterEmulator {
    pub smooth: f32,
    pub average_gain: f32,
    pub average_attenuation: f32,
    last_rms: (f32, f32),
    average_level: f32,
    max_level: f32,
    h: f32,
}

impl VUMeterEmulator {
    pub fn new(smooth: f32, average_gain: f32, average_attenuation: f32) -> Self {
        Self {
            smooth,
            average_gain,
            average_attenuation,
            last_rms: (0f32, 0f32),
            average_level: 0f32,
            max_level: 0f32,
            h: 0.0,
        }
    }

    pub fn process(&mut self, rms: (f32, f32), colors: &mut [Color]) {
        self.h += 0.05f32;
        if self.h > 1.0f32 {
            self.h = 0.0f32;
        }
        self.last_rms.0 = rms.0 * self.smooth - self.last_rms.0 * (1.0f32 - self.smooth);
        self.last_rms.1 = rms.1 * self.smooth - self.last_rms.1 * (1.0f32 - self.smooth);
        self.average_level = (self.last_rms.0 + self.last_rms.1) / 2.0f32
            * self.average_attenuation
            + self.average_level * (1.0f32 - self.average_attenuation);
        self.max_level = self.average_level * self.average_gain;

        let vu = Self::map(
            self.last_rms.0,
            0.0f32,
            self.max_level,
            0.0f32,
            colors.len() as f32,
        ) as usize;
        for (index, color) in colors.iter_mut().enumerate() {
            if index < vu {
                *color = to_rgb(self.h, 1.0f32, 1.0f32);
            } else {
                *color = Color::Rgb(32, 0, 0);
            }
        }
    }

    pub fn max(&self) -> f32 {
        self.max_level
    }

    pub fn average(&self) -> f32 {
        self.average_level
    }

    pub fn map(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
        (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
    }
}

impl Default for VUMeterEmulator {
    fn default() -> Self {
        Self::new(0.3f32, 1.6f32, 0.009f32)
    }
}

fn to_rgb(h: f32, s: f32, v: f32) -> Color {
    let range = (h / 60.0) as u8;
    let c = v * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = v - c;

    match range {
        0 => f32_rgb_to_color((c + m) * 255.0, (x + m) * 255.0, m * 255.0),
        1 => f32_rgb_to_color((x + m) * 255.0, (c + m) * 255.0, m * 255.0),
        2 => f32_rgb_to_color(m * 255.0, (c + m) * 255.0, (x + m) * 255.0),
        3 => f32_rgb_to_color(m * 255.0, (x + m) * 255.0, (c + m) * 255.0),
        4 => f32_rgb_to_color((x + m) * 255.0, m * 255.0, (c + m) * 255.0),
        _ => f32_rgb_to_color((c + m) * 255.0, m * 255.0, (x + m) * 255.0),
    }
}

fn f32_rgb_to_color(r: f32, g: f32, b: f32) -> Color {
    Color::Rgb(r as u8, g as u8, b as u8)
}
