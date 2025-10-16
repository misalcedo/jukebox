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