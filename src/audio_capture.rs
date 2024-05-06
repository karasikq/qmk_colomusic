use anyhow::Result;
use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, InputCallbackInfo, Sample, SizedSample,
    StreamConfig, StreamError,
};
use dasp_sample::ToSample;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

pub trait Processor: Send + Sync {
    fn process<S>(&mut self, data: &[S], info: &InputCallbackInfo, config: &StreamConfig)
    where
        S: SizedSample + ToSample<f32>;
    fn process_error(&mut self, err: StreamError);
    fn timeout(&self) -> Option<Duration>;
}

pub fn get_output_audio_devices() -> Vec<Device> {
    let mut result = Vec::new();
    let available_hosts = cpal::available_hosts();

    for host_id in available_hosts {
        let host = cpal::host_from_id(host_id).unwrap();
        let devices = host.devices().unwrap();
        for device in devices {
            result.push(device);
        }
    }

    result
}

pub fn get_default_audio_output_device() -> Option<Device> {
    let default_host = cpal::default_host();
    println!("{}", default_host.default_output_device().unwrap().name().unwrap());
    default_host.default_output_device()
}

pub fn capture_device_ouput<P>(device: &Device, processor: Arc<Mutex<P>>) -> Result<cpal::Stream>
where
    P: Processor + 'static,
{
    let supported_config = device.default_input_config()?;
    let config = supported_config.config();
    let move_config = config.clone();
    let move_processor = processor.clone();
    let stream = match supported_config.sample_format() {
        cpal::SampleFormat::I8 => device.build_input_stream(
            &config,
            move |data, info| {
                processor
                    .lock()
                    .unwrap()
                    .process::<i8>(data, info, &move_config)
            },
            move |err| move_processor.lock().unwrap().process_error(err),
            None,
        )?,
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data, info| {
                processor
                    .lock()
                    .unwrap()
                    .process::<i16>(data, info, &move_config)
            },
            move |err| move_processor.lock().unwrap().process_error(err),
            None,
        )?,
        cpal::SampleFormat::I32 => device.build_input_stream(
            &config,
            move |data, info| {
                processor
                    .lock()
                    .unwrap()
                    .process::<i32>(data, info, &move_config)
            },
            move |err| move_processor.lock().unwrap().process_error(err),
            None,
        )?,
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data, info| {
                processor
                    .lock()
                    .unwrap()
                    .process::<f32>(data, info, &move_config)
            },
            move |err| move_processor.lock().unwrap().process_error(err),
            None,
        )?,
        sample_format => {
            return Err(anyhow::Error::msg(format!(
                "Unsupported sample format '{sample_format}'"
            )))
        }
    };
    Ok(stream)
}

pub struct U8RmsProcessor {
    rms: (f32, f32),
}

impl U8RmsProcessor {
    pub fn new() -> Self {
        Self {
            rms: (0f32, 0f32),
        }
    }

    pub fn get_rms(&self) -> (u8, u8) {
        (self.rms.0.to_sample::<u8>(), self.rms.0.to_sample::<u8>())
    }
}

impl Default for U8RmsProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor for U8RmsProcessor {
    fn process<S>(&mut self, data: &[S], info: &InputCallbackInfo, config: &StreamConfig)
    where
        S: SizedSample + ToSample<f32>,
    {
        let mut sum = (0f32, 0f32);
        for frame in data.chunks_exact(2) {
            let samples = (frame[0].to_sample::<f32>(), frame[1].to_sample::<f32>());
            sum.0 += samples.0 * samples.0;
            sum.1 += samples.1 * samples.1;
        }

        let len_2 = (data.len() / 2) as f32;
        self.rms.0 = (sum.0 / len_2).sqrt();
        self.rms.1 = (sum.1 / len_2).sqrt();
        println!("{:?}", self.rms);
    }

    fn process_error(&mut self, err: StreamError) {}

    fn timeout(&self) -> Option<Duration> {
        None
    }
}
