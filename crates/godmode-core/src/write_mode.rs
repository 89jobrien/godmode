//! Explicit preview-versus-apply mode for mutating operations.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WriteMode {
    Preview,
    Apply,
}

impl WriteMode {
    pub fn is_apply(self) -> bool {
        matches!(self, Self::Apply)
    }
}
