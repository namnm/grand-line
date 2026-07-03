use crate::prelude::*;

#[field_names]
#[derive(Clone)]
pub struct AuthAttr {
    pub unauthenticated: bool,
}
impl TryFrom<Attr> for AuthAttr {
    type Error = SynErr;
    fn try_from(a: Attr) -> SynRes<Self> {
        Ok(Self {
            unauthenticated: a.bool_should_omit(Self::FIELD_UNAUTHENTICATED)?,
        })
    }
}
impl AttrValidate for AuthAttr {
    fn attr_fields(_attr: &Attr) -> Vec<String> {
        Self::FIELDS.iter().copied().map(|f| f.to_owned()).collect()
    }
}
