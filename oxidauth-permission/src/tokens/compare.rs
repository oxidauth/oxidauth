use super::*;

pub fn compare(input: &[Token<'_>], challenge: &[Token<'_>]) -> bool {
    use Token::*;
    
    if input.is_empty() || challenge.is_empty() {
        return false
    }

    let mut i = 0;
    let mut j = 0;

    while i < input.len() && j < challenge.len() {
        #[cfg(test)]
        if false {
            println!(
                "{}\n{}\n\n",
                underline_current(input, i),
                underline_current(challenge, j),
            );
        }

        match (
            (i == input.len() - 1 || j == challenge.len() - 1),
            compare_tokens(&input[i], &challenge[j]),
            &challenge[j],
        ) {
            (true, true, _) => break,
            (_, true, Double) => {
                if let Some(advance) = advance_till(&[Colon], &input[i..]) {
                    i += advance;
                } else {
                    return false;
                }

                if let Some(advance) = advance_till(&[Colon], &challenge[j..]) {
                    j += advance;
                } else {
                    return false;
                }
            },
            (_, true, Single) => {
                if let Some(advance) =
                    advance_till(&[Period, Colon], &input[i..])
                {
                    i += advance;
                } else {
                    return false;
                }

                if let Some(advance) = advance_till(
                    &[Period, Colon],
                    &challenge[j..],
                ) {
                    j += advance;
                } else {
                    return false;
                }
            },
            (_, true, _) => {
                i += 1;
                j += 1;
            },
            (_, false, _) => return false,
        }
    }

    true
}

fn compare_tokens(t1: &Token, t2: &Token) -> bool {
    use Token::*;

    match (t1, t2) {
        (t1, t2) if t1 == t2 => true,
        (_, t2) if t2 == &Double => true,
        (_, t2) if t2 == &Single => true,
        _ => false,
    }
}

fn advance_till(needles: &[Token], haystack: &[Token]) -> Option<usize> {
    haystack
        .iter()
        .position(|token| needles.contains(token))
}

#[cfg(test)]
use colored::Colorize;

#[cfg(test)]
fn underline_current(tokens: &[Token<'_>], position: usize) -> String {
    use std::fmt::Write;

    let mut buf = format!("{position}  ");

    for i in 0..tokens.len() {
        let s = format!("{:?}", tokens[i]);

        match i == position {
            true => write!(buf, "{}", s.underline()),
            false => write!(buf, "{}", s),
        }
        .unwrap();

        if i < tokens.len() {
            write!(buf, ", ").unwrap();
        }
    }

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_compare {
        ($b:expr, $set:expr, $challenge:expr) => {
            assert_eq!(
                $b,
                compare(&$set, &$challenge)
            );
        };
    }

    #[test]
    fn test_compare() {
        use Token::*;

        assert_compare!(
            true,
            [
                Dynamic("oxidauth"),
                Colon,
                Dynamic("users"),
                Colon,
                Dynamic("read")
            ],
            [Single, Colon, Double, Colon, Double]
        );

        assert_compare!(
            false,
            [
                Dynamic("oxidauth"),
                Colon,
                Dynamic("users"),
                Colon,
                Dynamic("read")
            ],
            [Dynamic("oxid"), Colon, Double, Colon, Double]
        );

        assert_compare!(
            true,
            [
                Dynamic("oxidauth"),
                Colon,
                Dynamic("users"),
                Colon,
                Dynamic("read")
            ],
            [Double, Colon, Double, Colon, Double]
        );

        assert_compare!(
            true,
            [
                Dynamic("oxidauth"),
                Period,
                Dynamic("admin"),
                Colon,
                Dynamic("users"),
                Colon,
                Dynamic("read")
            ],
            [
                Dynamic("oxidauth"),
                Period,
                Double,
                Colon,
                Dynamic("users"),
                Colon,
                Dynamic("read")
            ]
        );

        assert_compare!(
            true,
            [
                Dynamic("oxidauth"),
                Period,
                Dynamic("admin"),
                Colon,
                Dynamic("users"),
                Colon,
                Dynamic("read")
            ],
            [
                Dynamic("oxidauth"),
                Period,
                Single,
                Colon,
                Dynamic("users"),
                Colon,
                Dynamic("read")
            ]
        );

        assert_compare!(
            true,
            [
                Dynamic("oxidauth"),
                Period,
                Dynamic("admin"),
                Colon,
                Dynamic("users"),
                Period,
                Dynamic("1"),
                Colon,
                Dynamic("read")
            ],
            [
                Dynamic("oxidauth"),
                Period,
                Dynamic("admin"),
                Colon,
                Dynamic("users"),
                Period,
                Single,
                Colon,
                Dynamic("read")
            ]
        );

        assert_compare!(
            true,
            [
                Dynamic("oxidauth"),
                Period,
                Dynamic("admin"),
                Colon,
                Dynamic("users"),
                Colon,
                Dynamic("read")
            ],
            [Double, Colon, Double, Colon, Double]
        );

        assert_compare!(
            true,
            [
                Dynamic("oxidauth"),
                Period,
                Dynamic("admin"),
                Colon,
                Dynamic("users"),
                Colon,
                Dynamic("read")
            ],
            [Double, Colon, Double, Colon, Double]
        );

        assert_compare!(
            true,
            [
                Dynamic("oxidauth"),
                Period,
                Dynamic("admin"),
                Period,
                Dynamic("admin"),
                Colon,
                Dynamic("users"),
                Colon,
                Dynamic("read")
            ],
            [Double, Colon, Double, Colon, Double]
        );
        
        assert_compare!(
            false,
            [],
            [Double, Colon, Double, Colon, Double]
        );
        
        assert_compare!(
            false,
            [
                Dynamic("oxidauth"),
                Period,
                Dynamic("admin"),
                Period,
                Dynamic("admin"),
                Colon,
                Dynamic("users"),
                Colon,
                Dynamic("read")
            ],
            []
        );
    }

    #[test]
    fn test_advance_till() {
        use Token::*;

        assert_eq!(
            advance_till(
                &[Colon, Single],
                &[Single, Colon, Single, Colon]
            ),
            Some(0),
        );

        assert_eq!(
            advance_till(
                &[Colon],
                &[Single, Colon, Single, Colon]
            ),
            Some(1),
        );

        assert_eq!(
            advance_till(
                &[Single],
                &[Single, Colon, Single, Colon]
            ),
            Some(0),
        );

        assert_eq!(
            advance_till(
                &[Double],
                &[Single, Colon, Single, Colon]
            ),
            None,
        );
    }
}
