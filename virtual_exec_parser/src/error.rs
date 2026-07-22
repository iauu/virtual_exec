#[derive(Debug, Clone)]
pub enum ParseError {
    InconsistentIndentationError(usize),
    SynParseError(syn::Error),
    InvalidAssignmentTarget,
}
