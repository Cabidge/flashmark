pub use delimiter_macro_derive::InlineDelimiter;

/// Describes a left and right delimiters for a rule.
pub trait InlineDelimiter: 'static {
    const MARKER: char;
    const LEFT_DELIM: &'static str;
    const RIGHT_DELIM: &'static str = Self::LEFT_DELIM;
}

/// The default delimiter for inline math.
///
/// This delimiter looks like this: ``$`...`$``.
#[derive(InlineDelimiter)]
#[delimiter = "$`"]
pub struct DollarTickDelimiter;

/// An alternative delimiter for inline math.
///
/// This delimiter looks like this: `$...$`.
#[derive(InlineDelimiter)]
#[delimiter = '$']
pub struct DollarDelimiter;
