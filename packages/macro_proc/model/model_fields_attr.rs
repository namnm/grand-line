use crate::prelude::*;

pub struct ModelFieldsAttr {
    pub defaults: Vec<Attr>,
    pub virtuals: Vec<(Field, Vec<Attr>)>,
    pub exprs: Vec<(Field, Vec<Attr>)>,
    pub gql_fields: Vec<(Field, Vec<Attr>)>,
    pub sql_fields: FieldsNamed,
}

/// Parse macro attributes, extract and validate fields.
pub fn model_fields_attr(model: &str, fields: &Punctuated<Field, Token![,]>) -> SynRes<ModelFieldsAttr> {
    let (mut defaults, mut virtuals, mut exprs, mut gql, mut sql) = (vec![], vec![], vec![], vec![], vec![]);

    for f in fields {
        let attrs = Attr::from_field(model, f, &|a| ATTR_RAW.contains(a))?;
        attr_validate(&attrs)?;
        // default
        if let Some(def) = attrs.iter().find(|a| a.attr == AttrTy::Default) {
            defaults.push(def.clone());
        }
        // virtuals
        if let Some(v) = attr_is_virtual(&attrs) {
            if v == VirtualTy::SqlExpr {
                exprs.push((f.clone(), attrs));
            } else {
                virtuals.push((f.clone(), attrs));
            }
            continue;
        }
        // sql
        let mut extracted = f.clone();
        extracted.attrs = attr_sql(&attrs)?;
        sql.push(extracted);
        // gql
        let mut extracted = f.clone();
        extracted.attrs = attr_gql(&attrs)?;
        gql.push((extracted, attrs));
    }

    Ok(ModelFieldsAttr {
        defaults,
        virtuals,
        exprs,
        gql_fields: gql,
        sql_fields: FieldsNamed {
            brace_token: Default::default(),
            named: Punctuated::from_iter(sql),
        },
    })
}

/// Validate or return error.
fn attr_validate(attrs: &[Attr]) -> SynRes<()> {
    let Some(attr) = attrs.first() else {
        return Ok(());
    };
    // ensure it should not have more than one of our attributes
    let map = AttrTy::all()
        .iter()
        .map(|t| (t.to_string(), t.clone()))
        .collect::<HashMap<_, _>>();
    let mut matches = vec![];
    for a in attrs {
        if let Some(v) = map.get(&a.attr) {
            matches.push(v.clone());
        }
    }
    if matches.len() > 1 {
        let model = attr.field_model()?;
        let field = attr.field_name()?;
        let matches = matches.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ");
        let msg = format!("{model}.{field} should have only one between: {matches}");
        return Err(SynErr::new(attr.span, msg));
    }
    Ok(())
}

/// All virtual attributes.
/// If any of these attributes matched, we should extract the whole field out.
fn attr_is_virtual(attrs: &[Attr]) -> Option<VirtualTy> {
    let map = VirtualTy::all()
        .iter()
        .map(|t| (t.to_string(), t.clone()))
        .collect::<HashMap<_, _>>();
    attrs.iter().find_map(|a| map.get(&a.attr)).cloned()
}

/// Filter to only keep related attrs for the sql model.
/// If any of these attributes matched, we should removed them out of the field.
fn attr_sql(attrs: &[Attr]) -> SynRes<Vec<Attribute>> {
    let mut tobe_removed = AttrTy::all().iter().map(|f| f.to_string()).collect::<HashSet<_>>();
    tobe_removed.insert(AttrTy::Graphql.to_string());
    attrs
        .iter()
        .filter(|a| !tobe_removed.contains(&a.attr))
        .map(|a| a.field_attr())
        .collect()
}

/// Filter to only keep attrs relevant to the gql model field:
/// doc comments and #[graphql(...)] pass through, all others are dropped.
fn attr_gql(attrs: &[Attr]) -> SynRes<Vec<Attribute>> {
    let keep = hashset!["doc", "graphql"];
    attrs
        .iter()
        .filter(|a| keep.contains(a.attr.as_str()))
        .map(|a| a.field_attr())
        .collect()
}

/// Check if we should not include this field in the gql resolver, filter, order by..
/// This field can be in the gql model to support sql dep.
pub fn attr_is_gql_skip(attrs: &[Attr]) -> bool {
    attrs.iter().any(|a| a.is("graphql") && a.has("skip"))
}
