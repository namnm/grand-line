#[path = "./setup.rs"]
mod setup;
use setup::*;

// #[create(Note, check = authenticated)] calls AmWrapper::exec(ctx), which fills created_by_id
// from ctx.auth() via IntoAmCtx, this is the mechanism examples/saas's
// role_create.rs/user_in_role_create.rs/impersonate.rs rely on.
#[tokio::test]
async fn create_macro_sets_created_by_id_from_auth() -> Res<()> {
    let d = setup().await?;
    let user_id = ulid();
    let bearer = create_session(&d.tmp, &user_id).await?;

    let mut h = d.h;
    h.insert(H_AUTHORIZATION, bearer);
    let s = d.s.data(h).finish();

    let q = "
    mutation($title: String!) {
        noteCreate(data:{ title: $title }) {
            id
            title
        }
    }
    ";
    let v = value!({
        "title": "The Pattern",
    });
    exec_assert_ok(&s, q, Some(v)).await;

    let note = Note::find()
        .filter(NoteColumn::Title.eq("The Pattern"))
        .one_or_404(&d.tmp.db)
        .await?;
    pretty_eq!(
        note.created_by_id,
        Some(user_id),
        "created_by_id should be set from the authenticated session's user id",
    );

    d.tmp.drop().await
}

#[tokio::test]
async fn create_macro_requires_authentication() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(d.h).finish();

    let q = "
    mutation($title: String!) {
        noteCreate(data: { title: $title }) {
            id
        }
    }
    ";
    let v = value!({
        "title": "The Pattern",
    });
    exec_assert_err(&s, q, Some(v), &AuthErr::Unauthenticated).await?;

    d.tmp.drop().await
}
