#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

pub use grand_line::prelude::*;

#[model]
pub struct User {
    pub name: String,
    #[has_one]
    pub person: Person,
    #[has_many]
    pub aliases: Alias,
    #[many_to_many]
    pub orgs: Org,
}
#[detail(User)]
fn resolver() {}
#[search(User)]
fn resolver() {
    (None, None)
}
#[count(User)]
fn resolver() {
    None
}
#[delete(User)]
fn resolver() {}

#[model]
pub struct Person {
    pub gender: String,
    pub user_id: String,
    #[belongs_to]
    pub user: User,
}
#[detail(Person)]
fn resolver() {}

#[model]
pub struct Alias {
    pub name: String,
    pub user_id: String,
}

#[model]
pub struct Org {
    pub name: String,
}
#[model]
pub struct UserInOrg {
    pub user_id: String,
    pub org_id: String,
}

#[derive(Default, MergedObject)]
pub struct Query(UserDetailQuery, UserSearchQuery, UserCountQuery, PersonDetailQuery);
#[derive(Default, MergedObject)]
pub struct Mutation(UserDeleteMutation);

pub struct Prepare {
    pub tmp: TmpDb,
    pub s: GraphQLSchema<Query, Mutation, EmptySubscription>,
    pub id1: String,
    pub id2: String,
    pub pid1: String,
    pub pid2: String,
}

pub async fn prepare() -> Res<Prepare> {
    let tmp = tmp_db!(User, Person, Alias, Org, UserInOrg);
    let s = schema_qm::<Query, Mutation>(&tmp.db).finish();

    let u1 = am_create!(User {
        name: "Olivia",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let u2 = am_create!(User {
        name: "Peter",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    User::soft_delete_by_id(&u2.id)?.exec(&tmp.db).await?;

    let p1 = am_create!(Person {
        gender: "Female",
        user_id: u1.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let p2 = am_create!(Person {
        gender: "Male",
        user_id: u2.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    Person::soft_delete_by_id(&p1.id)?.exec(&tmp.db).await?;

    am_create!(Alias {
        name: "Liv",
        user_id: u1.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let a = am_create!(Alias {
        name: "Fauxlivia",
        user_id: u1.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    Alias::soft_delete_by_id(&a.id)?.exec(&tmp.db).await?;

    let o1 = am_create!(Org {
        name: "Fringe",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let o2 = am_create!(Org {
        name: "FBI",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    Org::soft_delete_by_id(&o2.id)?.exec(&tmp.db).await?;

    am_create!(UserInOrg {
        user_id: u1.id.clone(),
        org_id: o1.id,
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(UserInOrg {
        user_id: u1.id.clone(),
        org_id: o2.id,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    Ok(Prepare {
        tmp,
        s,
        id1: u1.id,
        id2: u2.id,
        pid1: p1.id,
        pid2: p2.id,
    })
}
