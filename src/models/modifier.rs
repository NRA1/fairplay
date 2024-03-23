use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum Modifier {
    Grayscale
}

impl Display for Modifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            match self { Modifier::Grayscale => { "Grayscale" } }
        )
    }
}