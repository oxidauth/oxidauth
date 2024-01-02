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
}

#[derive(Debug, PartialEq)]
enum Prev<'a> {
    Token(Token<'a>),
    Char,
    None,
}

pub fn validate(
    challenge: &[Token<'_>],
    permissions: &str,
) -> Result<bool, PermissionParseErr> {
    let parsed = parse::parse(permissions)?;

    let passed = compare::compare(challenge, &parsed);

    Ok(passed)
}
