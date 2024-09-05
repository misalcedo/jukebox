use anyhow::anyhow;
use pcsc::Context;
use std::ffi::CString;

pub struct Reader {
    ctx: Context,
    reader: CString,
}

impl Reader {
    pub fn new(ctx: Context, reader: CString) -> Reader {
        Reader { ctx, reader }
    }

    pub fn read(&self) -> anyhow::Result<String> {
        let card = match self.ctx.connect(&self.reader, pcsc::ShareMode::Direct, pcsc::Protocols::UNDEFINED) {
            Ok(card) => card,
            Err(pcsc::Error::NoSmartcard) => {
                return Err(anyhow!("A smartcard is not present in the reader."))?;
            }
            Err(err) => {
                return Err(anyhow!(err))?;
            }
        };

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

        Ok(String::new())
    }

    pub fn write(&self, value: String) -> anyhow::Result<()> {
        let card = self.ctx.connect(&self.reader, pcsc::ShareMode::Direct, pcsc::Protocols::UNDEFINED)?;

        Ok(())
    }
}
