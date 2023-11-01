#[derive(Clone, Copy)]
pub struct Directive<'a> {
    pub indent: usize,
    pub name: &'a str,
    pub args: Option<&'a str>,
}

pub struct MissingAtSignError;

impl<'a> TryFrom<&'a str> for Directive<'a> {
    type Error = MissingAtSignError;

    fn try_from(line: &'a str) -> Result<Self, Self::Error> {
        let trimmed = line.trim_start();
        let rest = trimmed.strip_prefix('@').ok_or(MissingAtSignError)?;

        let indent = line.len() - trimmed.len();

        let Some((name, args)) = rest.split_once(' ') else {
            return Ok(Directive {
                indent,
                name: rest,
                args: None,
            });
        };

        let args = Some(args.trim());

        Ok(Directive { indent, name, args })
    }
}
