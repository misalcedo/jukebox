use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::{Component, Path, PathBuf};
use rand::prelude::SliceRandom;
use walkdir::WalkDir;

pub struct Player {
    base_path: PathBuf,
    audio: Option<(OutputStream, Sink)>
}

impl Player {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path, audio: None }
    }

    pub async fn play(&mut self, uri: String) -> anyhow::Result<()> {
        // Strip the scheme and root path from the URI.
        // This forces the URI to be a relative path.
        let Some(file_path) = uri.strip_prefix("file:///") else {
            anyhow::bail!("Invalid URI")
        };

        let joined_path = normalize_path(self.base_path.join(file_path));
        if !joined_path.starts_with(&self.base_path) {
            return Err(anyhow::anyhow!("Invalid file path: {}", joined_path.display()));
        }

        let mut songs = Vec::new();
        for entry in WalkDir::new(&joined_path) {
            let dir_entry = entry?;
            if dir_entry.file_type().is_file() {
                songs.push(dir_entry.into_path());
            }
        }

        tracing::debug!(?songs, "Playing songs from {}", joined_path.display());

        if songs.is_empty() {
            return Ok(());
        }

        // Shuffle the songs to get a different order each time.
        songs.shuffle(&mut rand::rng());

        // Get an output stream handle to the default physical sound device.
        // Note that the playback stops when the stream_handle is dropped.
        let stream_handle =
            OutputStreamBuilder::open_default_stream()?;
        let sink = Sink::connect_new(stream_handle.mixer());

        for path in songs {
            let file = BufReader::new(File::open(&path)?);
            let source = Decoder::try_from(file)?;
            sink.append(source);
        }

        // The sound plays in a separate audio thread,
        // so we need to keep the main thread alive while it's playing.
        self.audio = Some((stream_handle, sink));

        Ok(())
    }

    pub async fn skip(&mut self) -> anyhow::Result<bool> {
        if let Some((_, sink)) = self.audio.as_mut() {
            sink.skip_one();
            sink.play();
            return Ok(true);
        }

        Ok(false)
    }

    pub async fn pause(&mut self) -> anyhow::Result<()> {
        if let Some((_, sink)) = self.audio.as_mut() {
            sink.pause();
        }

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
