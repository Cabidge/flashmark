use std::fmt;

/// A string that lazily escapes HTML characters when being formatted with Display.
#[derive(Debug, Clone, Hash)]
pub enum LazySanitize {
    Char(char),
    Static(&'static str),
    Owned(Box<str>),
}

pub fn escape_char(ch: char) -> Option<&'static str> {
    match ch {
        '<' => Some("&lt;"),
        '>' => Some("&gt;"),
        '&' => Some("&amp;"),
        '"' => Some("&quot;"),
        '\'' => Some("&#39;"),
        _ => None,
    }
}

pub fn write_escape_char(ch: char, f: &mut impl fmt::Write) -> fmt::Result {
    if let Some(s) = escape_char(ch) {
        f.write_str(s)
    } else {
        f.write_char(ch)
    }
}

impl fmt::Display for LazySanitize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LazySanitize::Char(ch) => {
                write_escape_char(*ch, f)?;
                return Ok(());
            }
            LazySanitize::Static(s) => s,
            LazySanitize::Owned(s) => s.as_ref(),
        };

        for ch in s.chars() {
            write_escape_char(ch, f)?;
        }

        Ok(())
    }
}

impl From<char> for LazySanitize {
    fn from(ch: char) -> Self {
        Self::Char(ch)
    }
}

impl From<&'static str> for LazySanitize {
    fn from(s: &'static str) -> Self {
        Self::Static(s)
    }
}

impl From<Box<str>> for LazySanitize {
    fn from(s: Box<str>) -> Self {
        Self::Owned(s)
    }
}

impl From<String> for LazySanitize {
    fn from(s: String) -> Self {
        Self::Owned(s.into_boxed_str())
    }
}
