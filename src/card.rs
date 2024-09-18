use anyhow::anyhow;
use pcsc::{Card, Context, ReaderState, State};
use std::ffi::CString;
use std::time::Duration;

pub struct Reader {
    ctx: Context,
    reader: CString,
    eject: bool,
}

// SW1 and SW2 for a successful operation.
const SUCCESS: &'static [u8; 2] = b"\x90\x00";

impl Reader {
    pub fn new(ctx: Context, reader: CString, eject: bool) -> Reader {
        Reader { ctx, reader, eject }
    }

    pub fn read(&self) -> anyhow::Result<Option<String>> {
        match self.connect()? {
            None => Ok(None),
            Some(card) => {
                let mut buffer = vec![0; 1024];

                let record_response = card.transmit(b"\xFF\xB0\x00\x04\x07", &mut buffer)?;

                let Some(record) = record_response.strip_suffix(SUCCESS) else {
                    return Err(anyhow!("The read operation failed."));
                };

                // Records seem to end in \xFE\x00

                let result = match record {
                    // No record
                    [3, 0, ..] => {
                        Ok(Some(String::new()))
                    }
                    // Empty record
                    [3, 4, b'\xD8', 0, 0, 0, ..] => {
                        Ok(Some(String::new()))
                    }
                    // URI record
                    [3, record_length, b'\xD1', 1, uri_length, b'\x55', b'\x04'] if *record_length >= 4 && *uri_length > 0 => {
                        eprintln!("{}: {:?}", record.len(), &record);

                        // let command = b"\xFF\xB0\x00\x04\x07";
                        // let response = card.transmit(command, &mut buffer)?;
                        // let Some(data) = response.strip_suffix(SUCCESS) else {
                        //     return Err(anyhow!("The read operation failed."));
                        // };

                        Ok(Some(String::new()))
                    }
                    _ => Ok(Some(String::new())),
                };

                if self.eject {
                    if let Err((_, e)) = card.disconnect(pcsc::Disposition::EjectCard) {
                        return Err(anyhow!(e));
                    }
                }

                result
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
                    let bytes = if i == blocks - 1 {
                        remaining
                    } else {
                        16
                    };

                    let length = u16::try_from(bytes)?.to_be_bytes();
                    let block = u8::try_from(i)?.to_be_bytes();

                    let mut command = Vec::with_capacity(6 + value.len());
                    command.extend_from_slice(b"\xFF\xD6\x00");
                    command.extend_from_slice(&block);
                    command.extend_from_slice(&length);
                    command.extend_from_slice(value.as_bytes());

                    println!("{:?}", card.transmit(&command, &mut vec![0; 1024])?);
                }


                if self.eject {
                    if let Err((_, e)) = card.disconnect(pcsc::Disposition::EjectCard) {
                        return Err(anyhow!(e));
                    }
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
                eprintln!("{:?}", card.transmit(&command, &mut vec![0; 2])?);

                if self.eject {
                    if let Err((_, e)) = card.disconnect(pcsc::Disposition::EjectCard) {
                        return Err(anyhow!(e));
                    }
                }

                Ok(true)
            }
        }
    }

    pub fn wait(&self, timeout: Option<Duration>) -> anyhow::Result<()> {
        let mut reader_states = [ReaderState::new(self.reader.clone(), State::UNAWARE)];

        while !reader_states[0].event_state().contains(State::PRESENT) {
            self.ctx.get_status_change(timeout, &mut reader_states)?;
        }

        Ok(())
    }

    fn connect(&self) -> anyhow::Result<Option<Card>> {
        match self.ctx.connect(
            &self.reader,
            pcsc::ShareMode::Shared,
            pcsc::Protocols::ANY,
        ) {
            Ok(card) => Ok(Some(card)),
            Err(pcsc::Error::NoSmartcard) => Ok(None),
            Err(e) => Err(anyhow!(e)),
        }
    }
}
