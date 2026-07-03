use crate::prelude::*;

#[field_names]
#[derive(Clone)]
pub struct AuthzAttr {
    pub realm: String,
    pub skip_org: bool,
    pub skip_user: bool,
}
impl TryFrom<Attr> for AuthzAttr {
    type Error = SynErr;
    fn try_from(a: Attr) -> SynRes<Self> {
        Ok(Self {
            realm: a.str_required(Self::FIELD_REALM)?,
            skip_org: a.bool_should_omit(Self::FIELD_SKIP_ORG)?,
            skip_user: a.bool_should_omit(Self::FIELD_SKIP_USER)?,
        })
    }
}
impl AttrValidate for AuthzAttr {
    fn attr_fields(_attr: &Attr) -> Vec<String> {
        Self::FIELDS.iter().copied().map(|f| f.to_owned()).collect()
    }
}
