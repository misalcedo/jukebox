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
                    return Err(anyhow!("The read operation failed for the record."));
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
                                    return Err(anyhow!("The read operation failed for data."));
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
                        eprintln!("Unknown record: {:?}", record);
                        Ok(Some(String::new()))
                    }
                }
            }
        }
    }

    pub fn write(&self, value: String) -> anyhow::Result<bool> {
        match self.connect()? {
            None => Ok(false),
            Some(card) => {
                let mut blocks = value.len() / 16;
                let remaining = value.len() % 16;

                if remaining > 0 {
                    blocks += 1;
                }

                for i in 0..blocks {
                    let bytes = if i == blocks - 1 { remaining } else { 16 };

                    let length = u16::try_from(bytes)?.to_be_bytes();
                    let block = u8::try_from(i)?.to_be_bytes();

                    let mut command = Vec::with_capacity(6 + value.len());
                    command.extend_from_slice(b"\xFF\xD6\x00");
                    command.extend_from_slice(&block);
                    command.extend_from_slice(&length);
                    command.extend_from_slice(value.as_bytes());

                    println!("{:?}", card.transmit(&command, &mut vec![0; 1024])?);
                }

                Ok(true)
            }
        }
    }

    pub fn erase(&self) -> anyhow::Result<bool> {
        match self.connect()? {
            None => Ok(false),
            Some(card) => {
                let value = [u8::MAX; 13];

                let block = u8::try_from(4)?.to_be_bytes();
                let length = u8::try_from(value.len())?.to_be_bytes();

                let mut command = Vec::with_capacity(5 + value.len());
                command.extend_from_slice(b"\xFF\xD6\x00");
                command.extend_from_slice(&block);
                command.extend_from_slice(&length);
                command.extend_from_slice(&value);

                eprintln!("{:?}", command);
                eprintln!("{:?}", card.transmit(&command, &mut [0; 2])?);

                Ok(true)
            }
        }
    }

    pub fn wait(&mut self, timeout: Option<Duration>) -> anyhow::Result<()> {
        let mut reader_states = [ReaderState::new(self.reader.clone(), self.state)];

        self.ctx.get_status_change(timeout, &mut reader_states)?;

        // Wait until the presence state toggles.
        while self.state.contains(State::PRESENT) == reader_states[0].event_state().contains(State::PRESENT) {
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
            Err(pcsc::Error::NoSmartcard) => Ok(None),
            Err(e) => Err(anyhow!(e)),
        }
    }
}
