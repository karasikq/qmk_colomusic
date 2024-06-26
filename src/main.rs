pub mod audio_capture;
pub mod protocol;
pub mod visualizer;

use std::{
    io::{self, Stdout},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    time::Duration,
};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use hidapi::HidApi;
use ratatui::prelude::*;

use crate::protocol::{Command, Protocol, ThreadCommand, PAGE_SIZE};

use self::{
    audio_capture::{capture_device_ouput, get_default_audio_output_device, RmsProcessor},
    visualizer::{LayoutWidget, VUMeterEmulator},
};

fn main() -> Result<()> {
    let processor = Arc::new(Mutex::new(RmsProcessor::new()));
    const VENDOR_ID: u16 = 0x19F5;
    const PRODUCT_ID: u16 = 0x3245;
    const USAGE_PAGE: u16 = 0xFF60;
    const USAGE: u16 = 0x61;

    let hidapi = HidApi::new()?;
    let device_info = hidapi
        .device_list()
        .find(|info| {
            info.product_id() == PRODUCT_ID
                && info.vendor_id() == VENDOR_ID
                && info.usage() == USAGE
                && info.usage_page() == USAGE_PAGE
        })
        .context("Cannot find keyboard device")?;

    println!(
        "Opening device:\n VID: {:04x}, PID: {:04x}\n",
        device_info.vendor_id(),
        device_info.product_id()
    );

    let hid_device = device_info.open_device(&hidapi)?;

    let (tx, rx): (Sender<ThreadCommand>, Receiver<ThreadCommand>) = mpsc::channel();

    let device = get_default_audio_output_device().unwrap();
    let _stream = capture_device_ouput(&device, processor.clone(), tx).unwrap();

    let protocol = Protocol::default();

    process_handshake(&hid_device, &protocol)?;

    let processor_hid = processor.clone();
    let raw_hid_handle = std::thread::spawn(move || -> Result<()> {
        hid_thread(&hid_device, &protocol, processor_hid, rx)
    });
    let mut terminal = setup_terminal().context("setup failed")?;
    run(&mut terminal, processor.clone()).context("app loop failed")?;
    restore_terminal(&mut terminal).context("restore terminal failed")?;

    raw_hid_handle.join().unwrap()?;
    Ok(())
}

fn process_handshake(hid_device: &hidapi::HidDevice, protocol: &Protocol) -> Result<()> {
    let handshake_command = Command::Handshake { status: 0x7F };
    println!("Handshaking with keyboard...");
    hid_device.write(&protocol.prepare_command(&handshake_command))?;
    let mut hid_buffer = [0; PAGE_SIZE];
    loop {
        let bytes = hid_device.read(&mut hid_buffer)?;
        println!("Response: {:?}", hid_buffer);
        let command = protocol.to_command(&hid_buffer[0..bytes])?;
        if let Command::Handshake { status } = command {
            if status == 0x80 {
                let handshake_command = Command::Handshake { status: 0x81 };
                println!("Received correct status. Sending confirmation...");
                hid_device.write(&protocol.prepare_command(&handshake_command))?;
                return Ok(());
            } else {
                println!("Received wrong value. Re-Handshaking with keyboard...");
                let handshake_command = Command::Handshake { status: 0x7F };
                hid_device.write(&protocol.prepare_command(&handshake_command))?;
            }
        }
    }
}

fn hid_thread(
    hid_device: &hidapi::HidDevice,
    protocol: &Protocol,
    processor: Arc<Mutex<RmsProcessor>>,
    rx: Receiver<ThreadCommand>,
) -> Result<()> {
    loop {
        let command = rx.recv().unwrap();
        match command {
            ThreadCommand::ProcessorComplete => {
                let rms = { processor.lock().unwrap().get_rms_u8() };
                let command = Command::RMS {
                    left: rms.0,
                    right: rms.1,
                };
                hid_device.write(&protocol.prepare_command(&command))?;
            }
        };
    }
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
