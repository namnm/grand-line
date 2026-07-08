use crate::prelude::*;

pub fn schema_q<Q>(db: &DatabaseConnection) -> SchemaBuilder<Q, EmptyMutation, EmptySubscription>
where
    Q: ObjectType + Default + 'static,
{
    let sb = GraphQLSchema::build(Q::default(), EmptyMutation, EmptySubscription);
    extension(sb, db)
}
pub fn schema_m<M>(db: &DatabaseConnection) -> SchemaBuilder<EmptyQuery, M, EmptySubscription>
where
    M: ObjectType + Default + 'static,
{
    let sb = GraphQLSchema::build(EmptyQuery::default(), M::default(), EmptySubscription);
    extension(sb, db)
}

pub fn schema_qm<Q, M>(db: &DatabaseConnection) -> SchemaBuilder<Q, M, EmptySubscription>
where
    Q: ObjectType + Default + 'static,
    M: ObjectType + Default + 'static,
{
    let sb = GraphQLSchema::build(Q::default(), M::default(), EmptySubscription);
    extension(sb, db)
}

fn extension<Q, M, S>(sb: SchemaBuilder<Q, M, S>, db: &DatabaseConnection) -> SchemaBuilder<Q, M, S>
where
    Q: ObjectType + 'static,
    M: ObjectType + 'static,
    S: SubscriptionType + 'static,
{
    sb.extension(GrandLineExtension).data(Arc::new(db.clone()))
}

#[derive(Default, SimpleObject)]
pub struct EmptyQuery {
    pub v: bool,
}
