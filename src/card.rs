use anyhow::anyhow;
use pcsc::{Card, Context, ReaderState, State};
use std::ffi::CString;
use std::time::Duration;

pub struct Reader {
    ctx: Context,
    reader: CString,
}

impl Reader {
    pub fn new(ctx: Context, reader: CString) -> Reader {
        Reader { ctx, reader }
    }

    pub fn read(&self) -> anyhow::Result<Option<String>> {
        match self.connect()? {
            None => Ok(None),
            Some(card) => {
                let command = b"\x00\xB0\x00\x00\x00\x00";
                let mut buffer = vec![0; 1024];

                let length = {
                    let response = card.transmit(command, &mut buffer)?;
                    response.len()
                };

                if let Err((_, err)) = card.disconnect(pcsc::Disposition::EjectCard) {
                    return Err(anyhow!(err));
                }

                buffer.truncate(length);

                let data = String::from_utf8(buffer)?;

                Ok(Some(data))
            }
        }
    }

    pub fn write(&self, value: String) -> anyhow::Result<bool> {
        match self.connect()? {
            None => Ok(false),
            Some(card) => {
                // length is variable from 1-3 bytes.
                // We only use 2 bytes as we don't need more than 64KB of data.
                let length = u16::try_from(value.len())?.to_be_bytes();

                let mut command = Vec::with_capacity(6 + value.len());
                command.extend_from_slice(b"\x00\xD0\x00\x00");
                command.extend_from_slice(&length);
                command.extend_from_slice(value.as_bytes());
                command.extend_from_slice(b"\x00");

                card.transmit(&command, &mut [])?;

                if let Err((_, err)) = card.disconnect(pcsc::Disposition::EjectCard) {
                    return Err(anyhow!(err));
                }

                Ok(true)
            }
        }
    }

    pub fn erase(&self) -> anyhow::Result<bool> {
        match self.connect()? {
            None => Ok(false),
            Some(card) => {
                let command = b"\x00\x0E\x00\x00\x00\x00";
                card.transmit(command, &mut [])?;

                if let Err((_, err)) = card.disconnect(pcsc::Disposition::EjectCard) {
                    return Err(anyhow!(err));
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
        match self.ctx.connect(&self.reader, pcsc::ShareMode::Direct, pcsc::Protocols::T0 | pcsc::Protocols::T1) {
            Ok(card) => Ok(Some(card)),
            Err(pcsc::Error::NoSmartcard) => {
                Ok(None)
            }
            Err(err) => {
                Err(anyhow!(err))
            }
        }
    }
}
