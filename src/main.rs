pub mod audio_capture;
pub mod visualizer;

use std::{
    io::{self, Stdout},
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use self::{
    audio_capture::{
        capture_device_ouput, get_default_audio_output_device, 
        RmsProcessor,
    },
    visualizer::{LayoutWidget, VUMeterEmulator},
};

fn main() -> Result<()> {
    let device = get_default_audio_output_device().unwrap();
    let processor = Arc::new(Mutex::new(RmsProcessor::new()));
    let _ = capture_device_ouput(&device, processor.clone()).unwrap();
    let mut terminal = setup_terminal().context("setup failed")?;
    run(&mut terminal, processor.clone()).context("app loop failed")?;
    restore_terminal(&mut terminal).context("restore terminal failed")?;

    // std::thread::sleep(ten_millis);
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode().context("failed to enable raw mode")?;
    execute!(stdout, EnterAlternateScreen).context("unable to enter alternate screen")?;
    Terminal::new(CrosstermBackend::new(stdout)).context("creating terminal failed")
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode().context("failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("unable to switch to main screen")?;
    terminal.show_cursor().context("unable to show cursor")
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    p: Arc<Mutex<RmsProcessor>>,
) -> Result<()> {
    let mut layout = visualizer::Layout::default();
    let mut vu_emulator = VUMeterEmulator::default();
    loop {
        terminal.draw(|f| {
            let processor = p.lock().unwrap();
            let rms = processor.get_rms::<f32>();
            let output = vu_emulator.process(rms);
            let vu =
                VUMeterEmulator::map(output.0, 0.0f32, vu_emulator.max(), 0.0f32, 84.0f32) as usize;
            for (index, color) in layout.colors.iter_mut().enumerate() {
                if index < vu {
                    *color = ratatui::style::Color::Rgb(255, 0, 0);
                } else {
                    *color = ratatui::style::Color::Rgb(32, 0, 0);
                }
            }
            let widget = LayoutWidget { layout: &layout };
            f.render_widget(widget, f.size());
        })?;

        if should_quit()? {
            break;
        }
    }
    Ok(())
}

fn should_quit() -> Result<bool> {
    if event::poll(Duration::from_millis(16)).context("event poll failed")? {
        if let Event::Key(key) = event::read().context("event read failed")? {
            return Ok(KeyCode::Char('q') == key.code);
        }
    }
    Ok(false)
}
