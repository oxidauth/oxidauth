use core::fmt;
use std::error::Error;

use self::parse::parse;

pub mod compare;
pub mod parse;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Token<'a> {
    Colon,
    Double,
    Period,
    Single,
    Dynamic(&'a str),
}

#[derive(Debug, PartialEq)]
pub enum PermissionParseErr {
    InvalidPermission,
    WildcardChallenge,
}

impl fmt::Display for PermissionParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionParseErr::InvalidPermission => write!(f, "invalid permission"),
            PermissionParseErr::WildcardChallenge => write!(f, "challenges don't support globs"),
        }
        
    }
}

impl Error for PermissionParseErr {}

#[derive(Debug, PartialEq)]
enum Prev<'a> {
    Token(Token<'a>),
    Char,
    None,
}

pub fn validate_single(
    challenge: &[Token<'_>],
    permissions: &str,
) -> Result<bool, PermissionParseErr> {
    let parsed = parse::parse(permissions)?;

    let passed = compare::compare(challenge, &parsed);

    Ok(passed)
}

pub fn validate(
    challenge: &[Token<'_>],
    permissions: &[impl AsRef<str>],
) -> Result<bool, PermissionParseErr> {
    for permission in permissions {
        match validate_single(challenge, permission.as_ref()) {
            Ok(true) => return Ok(true),
            Err(err) => return Err(err),
            Ok(false) => continue,
        }
    }

    Ok(false)
}

pub fn parse_and_validate(
    challenge: impl AsRef<str>,
    permissions: &[impl AsRef<str>],
) -> Result<bool, PermissionParseErr> {
    let challenge = parse(challenge.as_ref())?;
    
    if challenge.contains(&Token::Double) || challenge.contains(&Token::Single) {
        return Err(PermissionParseErr::WildcardChallenge)
    }

    validate(&challenge, permissions)
}

pub fn parse_and_validate_multiple(
    challenges: &[impl AsRef<str>],
    permissions: &[impl AsRef<str>],
) -> Result<bool, PermissionParseErr> {
    for challenge in challenges {
        match parse_and_validate(challenge, permissions) {
            Ok(true) => return Ok(true),
            Ok(false) => continue,
            Err(_) => continue,
        }
    }

    Ok(false)
}
