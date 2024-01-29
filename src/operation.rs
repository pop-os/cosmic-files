use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operation {
    /// Move a path to the trash
    Delete { path: PathBuf },
    /// Rename a path
    Rename { old: PathBuf, new: PathBuf },
    /// Restore a path from the trash
    Restore { path: PathBuf },
}

impl Operation {
    pub fn delete(path: impl Into<PathBuf>) -> Self {
        Self::Delete { path: path.into() }
    }

    pub fn rename(old: impl Into<PathBuf>, new: impl Into<PathBuf>) -> Self {
        Self::Rename {
            old: old.into(),
            new: new.into(),
        }
    }

    pub fn restore(path: impl Into<PathBuf>) -> Self {
        Self::Restore { path: path.into() }
    }

    pub fn reverse(self) -> Self {
        match self {
            Self::Delete { path } => Self::Restore { path },
            Self::Rename { old, new } => Self::Rename { old: new, new: old },
            Self::Restore { path } => Self::Delete { path },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Operation;

    #[test]
    fn operation() {
        assert_eq!(
            Operation::delete("foo").reverse(),
            Operation::restore("foo")
        );
        assert_eq!(
            Operation::rename("foo", "bar").reverse(),
            Operation::rename("bar", "foo")
        );
        assert_eq!(
            Operation::restore("foo").reverse(),
            Operation::delete("foo")
        );
    }
}
