// Parses command-line-like argument strings.
extern crate logos;

use logos::Logos;
use std::collections::HashMap;
use std::str::Chars;

#[derive(Logos)]
enum Tokens<'source> {
    #[regex("[\n\t \r]+", logos::skip)]
    #[error]
    Error,

    #[regex("\\-[a-zA-Z0-9]+", |lex| lex.slice()[1..].chars())]
    Flags(Chars<'source>),

    #[regex("\"[^\"]*\"", |lex| {let slice = lex.slice(); &slice[1..slice.len()-1]})]
    #[regex("[^\n\t \r\"-]+")]
    ArgStr(&'source str),

    #[regex("\\-\\-[a-zA-Z0-9]+", |lex| &lex.slice()[2..])]
    LongFlag(&'source str),
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Args<'a> {
    pub kwargs: HashMap<&'a str, &'a str>,
    pub args: Vec<&'a str>,
    pub flags: HashMap<char, u8>,
}

impl<'a> Args<'a> {
    pub fn parse(s: &'a str) -> Result<Self, ()> {
        let mut lexer = Tokens::lexer(s);
        let mut flags: HashMap<char, u8> = HashMap::new();
        let mut args: Vec<&str> = Vec::new();
        let mut kwargs: HashMap<&str, &str> = HashMap::new();
        while let Some(tok) = lexer.next() {
            match tok {
                Tokens::Flags(chars) => {
                    for c in chars {
                        *flags.entry(c).or_insert(0) += 1;
                    }
                }
                Tokens::ArgStr(arg) => args.push(arg),
                Tokens::LongFlag(flag) => {
                    if let Some(Tokens::ArgStr(arg)) = lexer.next() {
                        kwargs.insert(flag, arg);
                    } else {
                        return Err(());
                    }
                }
                _ => return Err(()),
            }
        }
        Ok(Args {
            kwargs,
            args,
            flags,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Args;
    use std::collections::HashMap;
    use std::iter::FromIterator;
    use std::default::Default;

    #[test]
    fn test_positional_simple() {
        assert_eq!(Args::parse("a b c d e"), Ok(Args {
            args: Vec::from(&["a", "b", "c", "d", "e"][..]), // Thanks Rust?
            ..Default::default()
        }));
    }

    #[test]
    fn test_positional_long() {
        assert_eq!(Args::parse(r#"a b "c d" e"#), Ok(Args {
            args: Vec::from(&["a", "b", "c d", "e"][..]),
            ..Default::default()
        }));
    }

    #[test]
    fn test_incomplete_long() {
        assert_eq!(Args::parse("a b \"c d"), Err(()));
    }

    #[test]
    fn test_kwargs() {
        assert_eq!(Args::parse(r#"--arg1 a --arg2 "b c""#), Ok(Args {
            kwargs: HashMap::from_iter([("arg1", "a"), ("arg2", "b c")].iter().copied()),
            ..Default::default()
        }));
    }

    #[test]
    fn test_incomplete_kwarg() {
        assert_eq!(Args::parse("--arg1 a --arg2"), Err(()));
    }

    #[test]
    fn test_flags() {
        assert_eq!(Args::parse("-flag -vvv"), Ok(Args {
            flags: HashMap::from_iter([('f', 1),('l', 1),('a',1),('g',1),('v',3)].iter().copied()),
            ..Default::default()
        }))
    }

    #[test]
    fn test_all() {
        assert_eq!(Args::parse(r#"short "long arg" --kw1 arg -vvv"#), Ok(Args {
            args: Vec::from(&["short", "long arg"][..]),
            kwargs: HashMap::from_iter([("kw1", "arg")].iter().copied()),
            flags: HashMap::from_iter([('v', 3)].iter().copied())
        }))
    }
}