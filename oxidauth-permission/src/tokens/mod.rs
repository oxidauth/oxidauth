pub mod compare;
pub mod parse;

#[derive(Debug, PartialEq)]
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
