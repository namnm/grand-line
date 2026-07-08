use crate::prelude::*;

#[field_names]
pub struct RelationAttr {
    pub include_deleted: bool,
    #[field_names(skip)]
    pub inner: Attr,
    #[field_names(virt)]
    key: !,
    #[field_names(virt)]
    through: !,
    #[field_names(virt)]
    other_key: !,
}
impl TryFrom<Attr> for RelationAttr {
    type Error = SynErr;
    fn try_from(a: Attr) -> SynRes<Self> {
        Ok(Self {
            include_deleted: a
                .bool(Self::FIELD_INCLUDE_DELETED)?
                .unwrap_or(FEATURE_RESOLVER_INCLUDE_DELETED),
            inner: a,
        })
    }
}
impl AttrValidate for RelationAttr {
    fn attr_fields(a: &Attr) -> Vec<String> {
        let mut f = vec![Self::FIELD_KEY];
        if a.attr == RelationTy::ManyToMany {
            f.push(Self::FIELD_THROUGH);
            f.push(Self::FIELD_OTHER_KEY);
        }
        f.iter().copied().map(|f| f.to_owned()).collect()
    }
}

impl RelationAttr {
    pub fn to(&self) -> SynRes<Ts2> {
        self.inner.field_ty()?.ts2_or_err()
    }
    pub fn gql_to(&self) -> SynRes<Ts2> {
        ty_gql(self.to()?)
    }
    pub fn name(&self) -> SynRes<Ts2> {
        self.inner.field_name()?.ts2_or_err()
    }

    pub fn key_str(&self) -> SynRes<String> {
        if let Some(v) = self.inner.str(Self::FIELD_KEY)? {
            return Ok(v);
        }
        let field = self.inner.field_name()?;
        let model = self.inner.field_model()?;
        Ok(if self.inner.attr == RelationTy::BelongsTo {
            format!("{field}_id").to_snake_case()
        } else {
            format!("{model}_id").to_snake_case()
        })
    }
    pub fn through(&self) -> SynRes<Ts2> {
        if let Some(v) = self.inner.str(Self::FIELD_THROUGH)? {
            return v.ts2_or_err();
        }
        let model = self.inner.field_model()?;
        let ty = self.inner.field_ty()?;
        if self.inner.attr != RelationTy::ManyToMany {
            return Err(self.bug(Self::FIELD_THROUGH));
        }
        format!("{model}_in_{ty}").to_pascal_case().ts2_or_err()
    }
    pub fn other_key(&self) -> SynRes<Ts2> {
        if let Some(v) = self.inner.str(Self::FIELD_OTHER_KEY)? {
            return v.ts2_or_err();
        }
        let ty = self.inner.field_ty()?;
        if self.inner.attr != RelationTy::ManyToMany {
            return Err(self.bug(Self::FIELD_OTHER_KEY));
        }
        format!("{ty}_id").to_snake_case().ts2_or_err()
    }

    fn bug(&self, k: &str) -> SynErr {
        let d = self.inner.attr_debug();
        let msg = format!("{d} key `{k}` should not access this key in this attr (programmer error)");
        SynErr::new(self.inner.span, msg)
    }
}
