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
                // Send an APDU command.
                let apdu = b"\x00\xa4\x04\x00\x0A\xA0\x00\x00\x00\x62\x03\x01\x0C\x06\x01";
                println!("Sending APDU: {:?}", apdu);
                let mut rapdu_buf = [0; 1024];
                let rapdu = match card.transmit(apdu, &mut rapdu_buf) {
                    Ok(rapdu) => rapdu,
                    Err(err) => {
                        eprintln!("Failed to transmit APDU command to card: {}", err);
                        std::process::exit(1);
                    }
                };
                println!("APDU response: {:?}", rapdu);
                Ok(Some(String::new()))
            }
        }
    }

    fn connect(&self) -> anyhow::Result<Option<Card>> {
        match self.ctx.connect(&self.reader, pcsc::ShareMode::Direct, pcsc::Protocols::UNDEFINED) {
            Ok(card) => Ok(Some(card)),
            Err(pcsc::Error::NoSmartcard) => {
                Ok(None)
            }
            Err(err) => {
                Err(anyhow!(err))
            }
        }
    }

    pub fn write(&self, value: String) -> anyhow::Result<()> {
        // Writes treat a missing card as an error.
        let card = self.ctx.connect(&self.reader, pcsc::ShareMode::Direct, pcsc::Protocols::UNDEFINED)?;

        Ok(())
    }

    pub fn wait(&self, timeout: Option<Duration>) -> anyhow::Result<()> {
        let mut reader_states = [ReaderState::new(self.reader.clone(), State::UNAWARE)];

        while !reader_states[0].event_state().contains(State::PRESENT) {
            self.ctx.get_status_change(timeout, &mut reader_states)?;
        }

        Ok(())
    }
}
