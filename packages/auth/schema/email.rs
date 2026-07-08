use crate::prelude::*;
use validator::ValidateEmail;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Email(pub String);

#[Scalar(name = "Email")]
impl ScalarType for Email {
    fn parse(value: GraphQLValue) -> InputValueResult<Self> {
        match value {
            GraphQLValue::String(mut s) => {
                s = s.trim().to_lowercase();
                if ValidateEmail::validate_email(&s) {
                    Ok(Self(s))
                } else {
                    Err(InputValueError::custom("Invalid email"))
                }
            }
            _ => Err(InputValueError::custom("Invalid email")),
        }
    }
    fn to_value(&self) -> GraphQLValue {
        GraphQLValue::String(self.0.clone())
    }
}
