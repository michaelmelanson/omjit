use nom_locate::LocatedSpan;

#[derive(Debug)]
pub struct SourceLocation<'a> {
    pub start: LocatedSpan<&'a str, ()>,
    pub end: LocatedSpan<&'a str, ()>,
}
