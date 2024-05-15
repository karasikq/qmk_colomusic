pub mod audio_capture;
pub mod visualizer;
pub mod protocol;

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
    audio_capture::{capture_device_ouput, get_default_audio_output_device, RmsProcessor},
    visualizer::{LayoutWidget, VUMeterEmulator},
};

fn main() -> Result<()> {
    let device = get_default_audio_output_device().unwrap();
    let processor = Arc::new(Mutex::new(RmsProcessor::new()));
    let _stream = capture_device_ouput(&device, processor.clone()).unwrap();
    let mut terminal = setup_terminal().context("setup failed")?;
    run(&mut terminal, processor.clone()).context("app loop failed")?;
    restore_terminal(&mut terminal).context("restore terminal failed")?;

    // std::thread::sleep(Duration::from_millis(10000));
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
            let rms = { p.lock().unwrap().get_rms::<f32>() };
            vu_emulator.process(rms, &mut layout.colors);
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
