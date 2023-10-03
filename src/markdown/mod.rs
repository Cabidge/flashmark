pub mod math;

/// Describes a left and right delimiters for a rule.
pub trait InlineDelimiter: 'static {
    const MARKER: char;
    const LEFT_DELIM: &'static str;
    const RIGHT_DELIM: &'static str = Self::LEFT_DELIM;
}

/// The default delimiter for inline math.
///
/// This delimiter looks like this: ``$`...`$``.
pub struct DollarTickDelimiter;

impl InlineDelimiter for DollarTickDelimiter {
    const MARKER: char = '$';
    const LEFT_DELIM: &'static str = "$`";
    const RIGHT_DELIM: &'static str = "`$";
}

/// An alternative delimiter for inline math.
///
/// This delimiter looks like this: `$...$`.
pub struct DollarDelimiter;

impl InlineDelimiter for DollarDelimiter {
    const MARKER: char = '$';
    const LEFT_DELIM: &'static str = "$";
}
