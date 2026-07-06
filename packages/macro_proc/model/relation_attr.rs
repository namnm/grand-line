use crate::prelude::*;

#[field_names]
pub struct RelationAttr {
    pub resolver: Option<Ident>,
    pub include_deleted: bool,
    pub count: bool,
    pub count_resolver: Option<Ident>,
    pub authz_row: bool,
    #[field_names(skip)]
    pub inner: Attr,
    #[field_names(key_only)]
    key: !,
    #[field_names(key_only)]
    through: !,
    #[field_names(key_only)]
    other_key: !,
}
impl TryFrom<Attr> for RelationAttr {
    type Error = SynErr;
    fn try_from(a: Attr) -> SynRes<Self> {
        let resolver = parse_resolver_ident(&a, Self::FIELD_RESOLVER, |field| format!("resolve_{field}"))?;
        let count_resolver =
            parse_resolver_ident(&a, Self::FIELD_COUNT_RESOLVER, |field| format!("resolve_{field}_count"))?;
        Ok(Self {
            resolver,
            include_deleted: a
                .bool(Self::FIELD_INCLUDE_DELETED)?
                .unwrap_or(FEATURE_RESOLVER_INCLUDE_DELETED),
            count: a.bool(Self::FIELD_COUNT)?.unwrap_or(FEATURE_RESOLVER_RELATION_COUNT),
            count_resolver,
            authz_row: a.bool(Self::FIELD_AUTHZ_ROW)?.unwrap_or(FEATURE_RESOLVER_AUTHZ_ROW),
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
        f.push(Self::FIELD_RESOLVER);
        if a.attr == RelationTy::HasMany || a.attr == RelationTy::ManyToMany {
            f.push(Self::FIELD_COUNT);
            f.push(Self::FIELD_COUNT_RESOLVER);
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

        let r = if self.inner.attr == RelationTy::BelongsTo {
            format!("{field}_id").to_snake_case()
        } else {
            format!("{model}_id").to_snake_case()
        };
        Ok(r)
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
        let msg = format!("{d} key {k}: should not access this key in this attr (programmer error)");
        SynErr::new(self.inner.span, msg)
    }
}

/// Parse a key that is either bare (true, default name) or a string (custom name),
/// e.g. resolver or resolver = "custom_name". default_name builds the default
/// ident from the field name, used when the key is bare or first_path.
fn parse_resolver_ident<F>(a: &Attr, key: &str, default_name: F) -> SynRes<Option<Ident>>
where
    F: Fn(&str) -> String,
{
    let r1 = a.bool(key);
    let r2 = a.str(key);
    let make_err = || {
        let msg = "must be true for default name or a string identifier for custom name";
        a.err_by_key(key, msg)
    };
    if r1.is_err() && r2.is_err() {
        return Err(make_err());
    }
    let make_default = || -> SynRes<Ident> {
        let field = a.field_name()?;
        Ok(format_ident!("{}", default_name(&field)))
    };
    let r = if a.first_path.clone().unwrap_or_default() == key {
        Some(make_default()?)
    } else if let Some(default) = r1.unwrap_or_default() {
        if !default {
            return Err(make_err());
        }
        Some(make_default()?)
    } else {
        r2?.map(|custom| format_ident!("{custom}"))
    };
    Ok(r)
}
