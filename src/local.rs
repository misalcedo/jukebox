use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;
use std::path::{Component, Path, PathBuf};

pub struct Player {
    base_path: PathBuf,
    output_stream: Option<OutputStream>
}

impl Player {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path, output_stream: None }
    }

    pub async fn play(&mut self, uri: String) -> anyhow::Result<()> {
        // TODO: implement full directory playback.

        // Strip the scheme and root path from the URI.
        // This forces the URI to be a relative path.
        let Some(file_path) = uri.strip_prefix("file:///") else {
            anyhow::bail!("Invalid URI")
        };

        let joined_path = normalize_path(self.base_path.join(file_path));
        anyhow::ensure!(joined_path.starts_with(&self.base_path), "Invalid file path: {}", joined_path.display());
        let file = BufReader::new(File::open(joined_path)?);

        // Get an output stream handle to the default physical sound device.
        // Note that the playback stops when the stream_handle is dropped.//!
        // TODO: need to work out how to keep the stream handle alive despite not being send.
        let stream_handle =
            rodio::OutputStreamBuilder::open_default_stream()?;
        // Load a sound from a file and decode that sound file into a source
        let source = Decoder::try_from(file)?;

        // TODO: use duration to play the entire directory.
        let duration = source.total_duration().unwrap_or_default();

        // Play the sound directly on the device
        stream_handle.mixer().add(source);

        // The sound plays in a separate audio thread,
        // so we need to keep the main thread alive while it's playing.
        self.output_stream = Some(stream_handle);

        tracing::debug!("Playing {} for {} seconds", uri, duration.as_secs());

        Ok(())
    }

    pub async fn pause(&mut self) -> anyhow::Result<()> {
        self.output_stream.take();
        Ok(())
    }
}

// From https://github.com/rust-lang/cargo/blob/fede83ccf973457de319ba6fa0e36ead454d2e20/src/cargo/util/paths.rs#L61
pub fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    let mut components = path.as_ref().components().peekable();
    let mut buffer = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                buffer.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                buffer.pop();
            }
            Component::Normal(c) => {
                buffer.push(c);
            }
        }
    }
    buffer
}
