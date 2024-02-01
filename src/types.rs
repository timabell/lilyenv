use crate::error::Error;

pub const PYPY_DOWNLOAD_URL: &str = "https://downloads.python.org/pypy/";

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Interpreter {
    CPython,
    PyPy,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Version {
    pub interpreter: Interpreter,
    pub major: u8,
    pub minor: u8,
    pub bugfix: Option<u8>,
}

impl Version {
    pub fn compatible(&self, other: &Self) -> bool {
        if self == other {
            true
        } else {
            self.interpreter == other.interpreter
                && self.major == other.major
                && self.minor == other.minor
                && other.bugfix.is_none()
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.interpreter {
            Interpreter::CPython => "",
            Interpreter::PyPy => "pypy",
        };
        match self.bugfix {
            Some(bugfix) => write!(f, "{}{}.{}.{}", prefix, self.major, self.minor, bugfix),
            None => write!(f, "{}{}.{}", prefix, self.major, self.minor),
        }
    }
}

impl std::str::FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match validate_version(s) {
            Ok((_, version)) => Ok(version),
            Err(_) => Err(Error::InvalidVersion(s.into())),
        }
    }
}

fn validate_version(version: &str) -> nom::IResult<&str, Version> {
    use nom::bytes::complete::tag;
    use nom::character::complete::u8;
    use nom::sequence::separated_pair;
    let (rest, interpreter) = nom::combinator::opt(tag("pypy"))(version)?;
    let (rest, (major, minor)) = separated_pair(u8, tag("."), u8)(rest)?;
    let (rest, bugfix) = nom::combinator::opt(nom::sequence::preceded(tag("."), u8))(rest)?;
    nom::combinator::eof(rest)?;
    let interpreter = match interpreter {
        Some(_) => Interpreter::PyPy,
        None => Interpreter::CPython,
    };
    Ok((
        rest,
        Version {
            interpreter,
            major,
            minor,
            bugfix,
        },
    ))
}

fn _parse_version(filename: &str) -> nom::IResult<&str, (String, Version)> {
    use nom::bytes::complete::tag;
    use nom::character::complete::u8;
    let (input, _) = tag("cpython-")(filename)?;
    let (input, (major, _, minor, _, bugfix, _, release_tag)) = nom::sequence::tuple((
        u8,
        tag("."),
        u8,
        tag("."),
        u8,
        tag("+"),
        nom::character::complete::digit1,
    ))(input)?;

    let version = Version {
        interpreter: Interpreter::CPython,
        major,
        minor,
        bugfix: Some(bugfix),
    };
    Ok((input, (release_tag.to_string(), version)))
}

pub fn parse_version(filename: &str) -> Result<(String, Version), Error> {
    match _parse_version(filename) {
        Ok((_, (release_tag, version))) => Ok((release_tag, version)),
        Err(_) => Err(Error::ParseAsset(filename.to_string())),
    }
}

fn _parse_pypy_version(url: &str) -> nom::IResult<&str, (String, String, Version)> {
    use nom::bytes::complete::{tag, take_until};
    use nom::character::complete::u8;
    let (filename, _) = tag(PYPY_DOWNLOAD_URL)(url)?;
    let (rest, (_, major, _, minor, _, release_tag)) =
        nom::sequence::tuple((tag("pypy"), u8, tag("."), u8, tag("-"), take_until("-")))(filename)?;

    let version = Version {
        interpreter: Interpreter::PyPy,
        major,
        minor,
        bugfix: None,
    };

    Ok((
        rest,
        (filename.to_string(), release_tag.to_string(), version),
    ))
}

pub fn parse_pypy_version(url: &str) -> Result<(String, String, Version), Error> {
    match _parse_pypy_version(url) {
        Ok((_, (filename, release_tag, version))) => Ok((filename, release_tag, version)),
        Err(_) => Err(Error::ParseAsset(url.to_string())),
    }
}
