use crate::prelude::*;

pub fn check_err<E>(r: &Response, e: &E) -> Res<()>
where
    E: GrandLineErrImpl,
{
    let Some(err) = &r.errors.first() else {
        return TestErr::expect("response should have an error");
    };

    pretty_eq!(err.message, e.to_string(), "error message should match");

    let Some(extensions) = err.extensions.as_ref() else {
        return TestErr::expect("error extensions should be some");
    };

    let Some(GraphQLValue::String(code)) = extensions.get("code") else {
        return TestErr::expect("error extensions code should be some string");
    };

    pretty_eq!(code, e.code(), "error extensions code should match");
    Ok(())
}
