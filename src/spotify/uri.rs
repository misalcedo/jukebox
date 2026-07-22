use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use url::Url;

#[derive(Debug, Clone)]
pub struct Uri {
    pub category: String,
    pub id: String,
}

impl Display for Uri {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "spotify:{}:{}", self.category, self.id)
    }
}

impl PartialEq<str> for Uri {
    fn eq(&self, other: &str) -> bool {
        let Some(("spotify", parts)) = other.split_once(":") else {
            return false;
        };
        let Some((category, id)) = parts.split_once(":") else {
            return false;
        };

        self.category == category && self.id == id
    }
}

impl PartialEq<&str> for Uri {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

#[derive(Debug)]
pub struct UriParseError;

impl Display for UriParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid URI")
    }
}

impl Error for UriParseError {}

impl FromStr for Uri {
    type Err = UriParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.split_once(":") {
            Some(("spotify", parts)) => {
                let (category, id) = parts.split_once(":").ok_or(UriParseError)?;

                Ok(Uri {
                    category: category.to_string(),
                    id: id.to_string(),
                })
            }
            Some(("https", _)) => {
                let url = Url::parse(s).map_err(|_| UriParseError)?;
                let mut path = url.path_segments().into_iter().flatten();

                match (path.next(), path.next()) {
                    (Some(category), Some(id)) => Ok(Uri {
                        category: category.to_string(),
                        id: id.to_string(),
                    }),
                    _ => Err(UriParseError),
                }
            }
            _ => Err(UriParseError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Uri;
    use std::str::FromStr;

    #[test]
    fn parses_spotify_uri() {
        let uri = Uri::from_str("spotify:track:123").unwrap();
        assert_eq!(uri.category, "track");
        assert_eq!(uri.id, "123");
        assert_eq!(uri.to_string(), "spotify:track:123");
    }

    #[test]
    fn parses_spotify_https_uri() {
        let uri = Uri::from_str("https://open.spotify.com/playlist/abc").unwrap();
        assert_eq!(uri.category, "playlist");
        assert_eq!(uri.id, "abc");
    }

    #[test]
    fn rejects_invalid_uri() {
        assert!(Uri::from_str("https://open.spotify.com/").is_err());
        assert!(Uri::from_str("spotify:track").is_err());
    }

    #[test]
    fn compares_with_str_uri() {
        let uri = Uri::from_str("spotify:album:999").unwrap();
        assert!(uri == "spotify:album:999");
        assert!(uri != "spotify:track:999");
    }
}