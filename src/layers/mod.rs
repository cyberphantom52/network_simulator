mod physical;

pub(self) enum Identifier {
    Name(String),
}

impl ToString for Identifier {
    fn to_string(&self) -> String {
        match self {
            Identifier::Name(name) => name.clone(),
        }
    }
}
