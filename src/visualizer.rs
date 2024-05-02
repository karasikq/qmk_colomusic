/*
⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿
⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿⣿⣿ ⣿
⣿⡇ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⢸⣿ ⣿
⣿⣧  ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⠨⣿⣿ ⣿
⣿⣿⠨  ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⣿ ⢸⣿ ⣿ ⣿
⣿ ⣿ ⣿         ⣿    ⣿ ⣿ ⣿ ⣿ ⣿  ⣿
*/

use std::fmt::Display;

use ratatui::{buffer::Buffer, layout::*, style::Style, text::Line, widgets::Widget};

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
    pub colors: [u8; 84],
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
            colors: [0; 84],
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
        let colors = &self.layout.colors;

        for (y, row) in self.layout.rows.iter().enumerate() {
            for key in row {
                let key_str: &'static str = key.into();
                let line = Line::from(key_str)
                    .style(Style::default().fg(ratatui::style::Color::Rgb(colors[absolute], 0, 0)));
                buf.set_line(area.x + x, area.y + y as u16, &line, line.width() as u16);
                x += line.width() as u16;
                absolute += 1;
            }
            x = 0;
        }
    }
}
