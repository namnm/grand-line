use crate::prelude::*;

pub fn check_err<E>(r: &Response, e: &E)
where
    E: GrandLineErrImpl,
{
    assert!(r.errors.len() == 1, "response should have an error");
    let Some(err) = &r.errors.first() else {
        return;
    };

    let expected_message = e.to_string();
    pretty_eq!(err.message, expected_message, "error message should match");

    if let Some(extensions) = err.extensions.as_ref() {
        let expected_code = e.code();
        if let Some(GraphQLValue::String(code)) = extensions.get("code") {
            pretty_eq!(code, expected_code, "error extensions code should match");
        } else {
            assert!(false, "error extensions code should be some string");
        }
    } else {
        assert!(false, "error extensions should be some");
    }
}
