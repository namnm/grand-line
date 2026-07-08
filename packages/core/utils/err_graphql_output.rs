use crate::prelude::*;

#[derive(Clone)]
pub struct GrandLineErr(pub Arc<dyn GrandLineErrImpl>);
pub type Res<T> = Result<T, GrandLineErr>;

impl Display for GrandLineErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtRes {
        Display::fmt(&self.0, f)
    }
}
impl Debug for GrandLineErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtRes {
        Debug::fmt(&self.0, f)
    }
}
impl Error for GrandLineErr {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

impl From<GrandLineErr> for ServerError {
    fn from(v: GrandLineErr) -> Self {
        let mut err = Self::new(v.to_string(), None);
        err.source = Some(Arc::new(v));
        err
    }
}
