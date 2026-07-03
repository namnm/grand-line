use crate::prelude::*;

#[field_names]
pub struct ModelAttr {
    pub created_at: bool,
    pub updated_at: bool,
    pub deleted_at: bool,
    pub by_id: bool,
    #[field_names(skip)]
    pub inner: Attr,
}
impl TryFrom<Attr> for ModelAttr {
    type Error = SynErr;
    fn try_from(a: Attr) -> SynRes<Self> {
        Ok(Self {
            created_at: a.bool(Self::FIELD_CREATED_AT)?.unwrap_or(FEATURE_MODEL_CREATED_AT),
            updated_at: a.bool(Self::FIELD_UPDATED_AT)?.unwrap_or(FEATURE_MODEL_UPDATED_AT),
            deleted_at: a.bool(Self::FIELD_DELETED_AT)?.unwrap_or(FEATURE_MODEL_DELETED_AT),
            by_id: a.bool(Self::FIELD_BY_ID)?.unwrap_or(FEATURE_MODEL_BY_ID),
            inner: a,
        })
    }
}
impl AttrValidate for ModelAttr {
    fn attr_fields(_attr: &Attr) -> Vec<String> {
        Self::FIELDS.iter().copied().map(|f| f.to_owned()).collect()
    }
}
