use anyhow::anyhow;
use pcsc::{Card, Context, ReaderState, State};
use std::ffi::CString;
use std::time::Duration;

// The URI prefix for NFC tags.
const HTTPS_PREFIX: &[u8] = b"https://";

// SW1 and SW2 for a successful operation.
const SUCCESS: &[u8; 2] = b"\x90\x00";
// Number of bytes in a block.
const BLOCK_SIZE: u8 = b'\x04';
// Maximum number of bytes to read in a single operation.
const MAX_READ_BYTES: u8 = b'\x10';
// The first block with user data.
const INITIAL_DATA_BLOCK: u8 = b'\x04';

pub struct Reader {
    ctx: Context,
    reader: CString,
    state: State,
}

impl Reader {
    pub fn new(ctx: Context, reader: CString) -> Reader {
        Reader {
            ctx,
            reader,
            state: State::UNAWARE,
        }
    }

    pub fn read(&self) -> anyhow::Result<Option<String>> {
        match self.connect()? {
            None => Ok(None),
            Some(card) => {
                let mut buffer = vec![0; 1024];

                let record_response = card.transmit(b"\xFF\xB0\x00\x04\x07", &mut buffer)?;

                let Some(record) = record_response.strip_suffix(SUCCESS) else {
                    return Err(anyhow!("The read operation failed for the record"));
                };

                match record {
                    // No record
                    [3, 0, ..] => Ok(Some(String::new())),
                    // Empty record
                    [3, 4, b'\xD8', 0, 0, 0, ..] => Ok(Some(String::new())),
                    // URI record (single or the first of multiple)
                    [3, record_length, b'\xD1' | b'\x91', 1, uri_length, b'\x55', prefix]
                    if *record_length >= 4 && *uri_length > 0 =>
                        {
                            let mut bytes_read = record.len();
                            let mut remaining = *uri_length - 1;
                            let mut command = [
                                b'\xFF',
                                b'\xB0',
                                b'\x00',
                                INITIAL_DATA_BLOCK,
                                MAX_READ_BYTES,
                            ];

                            let uri_prefix = match prefix {
                                b'\x04' => HTTPS_PREFIX,
                                _ => b"",
                            };

                            let mut data = Vec::with_capacity(uri_prefix.len() + remaining as usize);
                            data.extend_from_slice(uri_prefix);

                            while remaining > 0 {
                                let offset = u8::try_from(bytes_read)?;
                                let block = INITIAL_DATA_BLOCK + (offset / BLOCK_SIZE);

                                // Update the block
                                command[3] = block;

                                // Update the requested bytes
                                command[4] = remaining.min(MAX_READ_BYTES);

                                let data_response = card.transmit(&command, &mut buffer)?;
                                let Some(mut chunk) = data_response.strip_suffix(SUCCESS) else {
                                    return Err(anyhow!("The read operation failed for data"));
                                };

                                // Skip already read bytes
                                let skip = (offset % BLOCK_SIZE) as usize;
                                if skip != 0 {
                                    chunk = &chunk[skip..];
                                }

                                remaining -= u8::try_from(chunk.len())?;
                                bytes_read += chunk.len();

                                data.extend_from_slice(chunk);
                            }

                            Ok(Some(String::from_utf8(data)?))
                        }
                    _ => {
                        tracing::warn!(record = format!("{:?}", record), "Unknown record");
                        Ok(Some(String::new()))
                    }
                }
            }
        }
    }

    pub fn wait(&mut self, timeout: Option<Duration>) -> anyhow::Result<()> {
        let mut reader_states = [ReaderState::new(self.reader.clone(), self.state)];

        self.ctx.get_status_change(timeout, &mut reader_states)?;

        // Wait until the presence state toggles.
        while self.state.contains(State::PRESENT)
            == reader_states[0].event_state().contains(State::PRESENT)
        {
            self.ctx.get_status_change(timeout, &mut reader_states)?;
        }

        self.state = reader_states[0].event_state();

        Ok(())
    }

    fn connect(&self) -> anyhow::Result<Option<Card>> {
        match self
            .ctx
            .connect(&self.reader, pcsc::ShareMode::Shared, pcsc::Protocols::ANY)
        {
            Ok(card) => Ok(Some(card)),
            Err(pcsc::Error::NoSmartcard | pcsc::Error::RemovedCard) => Ok(None),
            Err(e) => Err(anyhow!(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pcsc::ctl_code;

    #[test]
    fn get_led_and_buzzer() -> anyhow::Result<()> {
        let ctx = Context::establish(pcsc::Scope::User).expect("Failed to establish context");
        let reader = CString::new("ACS ACR1252 Dual Reader PICC")?;
        let card = ctx.connect(&reader, pcsc::ShareMode::Direct, pcsc::Protocols::ANY)?;

        let mut buffer = vec![0; 1024];
        let response = card.control(ctl_code(0x310000 + 3500 * 4), b"\xE0\x00\x00\x18\x00", &mut buffer)?;
        // let response = card.control(0x310000 + 3500 * 4, b"\xE0\x00\x00\x18\x00", &mut buffer)?;
        // let response = card.control(ctl_code(3500), b"\xE0\x00\x00\x21\x00", &mut buffer)?;

        assert_eq!(format!("{:X?}", response), String::new());

        Ok(())
    }

    #[test]
    fn set_led_and_buzzer() {
        let ctx = Context::establish(pcsc::Scope::User).unwrap();
        let reader = CString::new("ACS ACR1252 1S CL Reader PICC 0").unwrap();
        let card = ctx.connect(&reader, pcsc::ShareMode::Direct, pcsc::Protocols::UNDEFINED).unwrap();

        // Disable buzzer in all cases, but keep the other settings as the default.
        let command = [0xE0, 0x00, 0x00, 0x21, 0x01, 0b01000101];

        let mut buffer = vec![0; 1024];
        let response = card.control(ctl_code(3500), &command, &mut buffer).unwrap();

        assert_eq!(format!("{:X?}", response), String::from("[E1, 0, 0, 0, 1, 45]"));
    }
}