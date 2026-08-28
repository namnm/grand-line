use super::prelude::*;

/// Access to the per-request GrandLineData stored in the graphql context.
pub trait GrandLineDataContext<'a>
where
    Self: BaseImplContext<'a>,
{
    /// Returns the per-request GrandLineData, err Ctx404 if it was not inserted into context.
    fn grand_line(&self) -> Res<&'a GrandLineData> {
        let gl = self.data_opt_impl::<Arc<GrandLineData>>().ok_or(MyErr::Ctx404)?;
        Ok(gl)
    }

    /// The same data as a shared handle, for the few places that must outlive one
    /// borrow, e.g. a dataloader batch task, which holds a Weak of this.
    fn grand_line_arc(&self) -> Res<&'a Arc<GrandLineData>> {
        let gl = self.data_opt_impl::<Arc<GrandLineData>>().ok_or(MyErr::Ctx404)?;
        Ok(gl)
    }
}

impl<'a> GrandLineDataContext<'a> for Context<'a> {
}

impl<'a> GrandLineDataContext<'a> for ExtensionContext<'a> {
}
