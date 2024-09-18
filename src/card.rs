use anyhow::anyhow;
use pcsc::{Card, Context, ReaderState, State};
use std::ffi::CString;
use std::time::Duration;

pub struct Reader {
    ctx: Context,
    reader: CString,
    eject: bool,
}

impl Reader {
    pub fn new(ctx: Context, reader: CString, eject: bool) -> Reader {
        Reader { ctx, reader, eject }
    }

    pub fn read(&self) -> anyhow::Result<Option<String>> {
        match self.connect()? {
            None => Ok(None),
            Some(card) => {
                let command = b"\xFF\xB0\x00\x04\x10";
                let mut buffer = vec![0; 1024];

                let response = card.transmit(command, &mut buffer)?;
                let length = response.len();

                if self.eject {
                    if let Err((_, e)) = card.disconnect(pcsc::Disposition::EjectCard) {
                        return Err(anyhow!(e));
                    }
                }

                buffer.truncate(length);

                println!("{:?}", buffer);

                let data = String::from_utf8_lossy(&buffer).to_string();

                Ok(Some(data))
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
