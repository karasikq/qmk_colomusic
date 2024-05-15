pub enum ThreadCommand {
    ProcessorComplete,
}

pub struct Protocol {}

impl Protocol {
    pub fn new() -> Protocol {
        Self {}
    }

    fn header(&self) -> [u8; 4] {
        [0, b'k', b'b', b'm']
    }

    pub fn prepare_rms_data(&self, rms_left: u8, rms_right: u8) -> [u8; 33] {
        let mut data: [u8; 33] = [0; 33];
        let header = self.header();
        let (header_chunk, data_chunk) = data.split_at_mut(header.len());
        header_chunk.copy_from_slice(&header);
        data_chunk[0] = rms_left;
        data_chunk[1] = rms_right;
        data
    }
}

impl Default for Protocol {
    fn default() -> Self {
        Self::new()
    }
}
