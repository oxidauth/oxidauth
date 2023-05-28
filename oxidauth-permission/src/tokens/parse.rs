use super::*;

pub fn parse(s: &str) -> Result<Vec<Token<'_>>, PermissionParseErr> {
    use Token::*;

    let mut result = Vec::new();
    let mut colon_count = 0;
    let mut word_start = None;
    let mut prev = Prev::None;

    let mut it = s.char_indices().peekable();

    while let Some((idx, char)) = it.next() {
        match char {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => {
                if word_start.is_none() {
                    word_start = Some(idx);
                }

                prev = Prev::Char;
            }
            ':' => {
                match prev {
                    Prev::Char => {
                        if let Some(word_start) = word_start {
                            result.push(Dynamic(&s[word_start..idx]));
                        }
                        word_start = None;
                    }
                    Prev::None | Prev::Token(Period) | Prev::Token(Colon) => {
                        return Err(PermissionParseErr::InvalidPermission)
                    }
                    _ => {}
                }

                colon_count += 1;

                if colon_count > 2 {
                    return Err(PermissionParseErr::InvalidPermission);
                }

                prev = Prev::Token(Colon);
                result.push(Colon);
            }
            '*' => {
                match prev {
                    Prev::Char => {
                        if let Some(word_start) = word_start {
                            result.push(Dynamic(&s[word_start..idx]));
                        }
                        word_start = None;
                    }
                    Prev::Token(Single) | Prev::Token(Double) => {
                        return Err(PermissionParseErr::InvalidPermission)
                    }
                    _ => {}
                }

                if let Some((_, next)) = it.peek() {
                    if next == &'*' {
                        it.next();

                        prev = Prev::Token(Double);
                        result.push(Double);
                    } else {
                        prev = Prev::Token(Single);
                        result.push(Single);
                    }
                } else {
                    prev = Prev::Token(Single);
                    result.push(Single);
                }
            }
            '.' => {
                match prev {
                    Prev::Char => {
                        if let Some(word_start) = word_start {
                            result.push(Dynamic(&s[word_start..idx]));
                        }
                        word_start = None;
                    }
                    Prev::None | Prev::Token(Period) | Prev::Token(Colon) => {
                        return Err(PermissionParseErr::InvalidPermission)
                    }
                    _ => {}
                }

                prev = Prev::Token(Period);
                result.push(Period);
            }

            _ => return Err(PermissionParseErr::InvalidPermission),
        }
    }

    if let Some(start) = word_start {
        result.push(Dynamic(&s[start..s.len()]));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use Token::*;

    macro_rules! assert_permission {
        ($input:expr, [ $($result:expr),+ ]) => {
            let p = $input;
            let expected = Ok(vec![$($result),+]);

            let result = parse(p);

            assert_eq!(result, expected);
        };
        ($input:expr, error) => {
            let p = $input;

            let result = parse(p);

            assert!(result.is_err());
        };
    }

    #[test]
    fn parse_success() {
        assert_permission!(
            "a:b:c",
            [Dynamic("a"), Colon, Dynamic("b"), Colon, Dynamic("c")]
        );

        assert_permission!(
            "*:*.*:*",
            [Single, Colon, Single, Period, Single, Colon, Single]
        );

        assert_permission!("*:*:*", [Single, Colon, Single, Colon, Single]);

        assert_permission!("**:**:**", [Double, Colon, Double, Colon, Double]);

        assert_permission!(
            "realm:resource:action",
            [
                Dynamic("realm"),
                Colon,
                Dynamic("resource"),
                Colon,
                Dynamic("action")
            ]
        );

        assert_permission!("*:b:c", [Single, Colon, Dynamic("b"), Colon, Dynamic("c")]);

        assert_permission!(
            "oxidauth.admin_web.super_admin.tenant.special:tenant.c2fd240c-4160-459e-a184-084ddba63a94.users.c2fd240c-4160-459e-a184-084ddba63a94.relationship.c2fd240c-4160-459e-a184-084ddba63a94:read_manage_write_all.*",
            [Dynamic("oxidauth"), Period, Dynamic("admin_web"), Period, Dynamic("super_admin"), Period, Dynamic("tenant"), Period, Dynamic("special"), Colon, Dynamic("tenant"), Period, Dynamic("c2fd240c-4160-459e-a184-084ddba63a94"), Period, Dynamic("users"), Period, Dynamic("c2fd240c-4160-459e-a184-084ddba63a94"), Period, Dynamic("relationship"), Period, Dynamic("c2fd240c-4160-459e-a184-084ddba63a94"), Colon, Dynamic("read_manage_write_all"), Period, Single]
        );
    }

    #[test]
    fn parse_fail() {
        assert_permission!(".:b:c", error);

        assert_permission!("*:b.:c", error);

        assert_permission!("a:b:c:d", error);

        assert_permission!(" :b.:c", error);

        assert_permission!(":b.:c", error);

        assert_permission!("a:*:.", error);

        assert_permission!("a:***:c", error);

        assert_permission!(".:b:c", error);
    }
}
