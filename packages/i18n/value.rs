use crate::prelude::*;

// ---------------------------------------------------------------------------
// IntlValue -- generic dynamic value, Rhai-independent
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub enum IntlValue {
    Str(String),
    Int(i64),
    Float(f64),
}

impl IntlValue {
    pub const fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            Self::Float(f) => Some(*f as i64),
            Self::Str(_) => None,
        }
    }

    pub const fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            Self::Int(i) => Some(*i as f64),
            Self::Str(_) => None,
        }
    }
}

impl Display for IntlValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtRes {
        match self {
            Self::Str(s) => f.write_str(s),
            Self::Int(i) => write!(f, "{i}"),
            Self::Float(n) => write!(f, "{n}"),
        }
    }
}
