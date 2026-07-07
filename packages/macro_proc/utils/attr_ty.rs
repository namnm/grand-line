use crate::prelude::*;
use strum_macros::{AsRefStr, Display};

/// Which macro (model or one of the crud kinds) is currently being expanded.
#[derive(Clone, Eq, PartialEq, AsRefStr, Display, PartialEqString)]
#[strum(serialize_all = "snake_case")]
pub enum MacroTy {
    Model,
    Search,
    Count,
    Detail,
    Create,
    Update,
    Delete,
}

/// Field-level attribute names whose value is kept as a raw token stream
/// instead of being parsed, e.g. #[default(..)] and #[sql_expr(..)].
pub static ATTR_RAW: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    set.insert(AttrTy::Default.to_string());
    set.insert(VirtualTy::SqlExpr.to_string());
    set
});

/// A field-level attribute recognized inside #[model], either a plain
/// #[graphql(..)]/#[default(..)] or a virtual (relation/sql_expr/resolver) field.
#[derive(Clone, Eq, PartialEq, AsRefStr, Display, PartialEqString)]
#[strum(serialize_all = "snake_case")]
pub enum AttrTy {
    Graphql,
    Default,
    #[strum(serialize = "{0}")]
    Virtual(VirtualTy),
}
impl AttrTy {
    /// All recognized attribute names, virtual kinds first, Default last.
    pub fn all() -> Vec<Self> {
        let mut all = VirtualTy::all()
            .iter()
            .map(|r| Self::Virtual(r.clone()))
            .collect::<Vec<_>>();
        all.push(Self::Default);
        all
    }
}

/// A field that has no backing db column, its value is computed instead of stored.
#[derive(Clone, Eq, PartialEq, AsRefStr, Display, PartialEqString)]
#[strum(serialize_all = "snake_case")]
pub enum VirtualTy {
    #[strum(serialize = "{0}")]
    Relation(RelationTy),
    SqlExpr,
    Resolver,
}
impl VirtualTy {
    /// All virtual field kinds, every relation kind followed by SqlExpr and Resolver.
    pub fn all() -> Vec<Self> {
        let mut all = RelationTy::all()
            .iter()
            .map(|r| Self::Relation(r.clone()))
            .collect::<Vec<_>>();
        all.push(Self::SqlExpr);
        all.push(Self::Resolver);
        all
    }
}

/// Kind of relation a #[relation(..)] virtual field describes.
#[derive(Clone, Eq, PartialEq, AsRefStr, Display, PartialEqString)]
#[strum(serialize_all = "snake_case")]
pub enum RelationTy {
    BelongsTo,
    HasOne,
    HasMany,
    ManyToMany,
}
impl RelationTy {
    /// All relation kinds.
    pub fn all() -> Vec<Self> {
        vec![Self::BelongsTo, Self::HasOne, Self::HasMany, Self::ManyToMany]
    }
}
