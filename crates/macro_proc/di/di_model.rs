use crate::prelude::*;

/// The #[model] expansion a di macro is attached to. Those macros sit directly
/// under #[model], where the annotated item is already the generated Model
/// struct inside the entity module, so the impls they emit target Entity and
/// Column from that same module.
pub struct DiModel {
    item: ItemStruct,
    name: String,
    fields: HashSet<String>,
    span: Span,
}

impl DiModel {
    /// Parses the annotated item, erroring when the macro is not under #[model].
    pub fn parse(name: &str, item: TokenStream) -> SynRes<Self> {
        let item = parse::<ItemStruct>(item)?;
        let span = item.ident.span();
        if item.ident != "Model" {
            let msg = format!("{name} should be placed under #[model]");
            return Err(SynErr::new(span, msg));
        }
        let fields = item
            .fields
            .iter()
            .map(|f| f.ident.to_token_stream().to_string())
            .collect();
        Ok(Self {
            item,
            name: name.to_owned(),
            fields,
            span,
        })
    }

    /// Errors when the model is missing any field the generated impl reads.
    pub fn require(&self, fields: &[&str]) -> SynRes<()> {
        for f in fields {
            if !self.fields.contains(*f) {
                let name = &self.name;
                let msg = format!("{name} requires the model to have field: {f}");
                return Err(SynErr::new(self.span, msg));
            }
        }
        Ok(())
    }

    /// Re-emits the annotated item untouched, followed by the generated impls.
    pub fn g(&self, impls: &Ts2) -> TokenStream {
        let item = &self.item;
        let r = quote! {
            #item
            #impls
        };

        #[cfg(feature = "debug_macro")]
        debug_macro(&self.name, &r);

        r.into()
    }
}

/// Errors when a di macro taking no argument was given one.
pub fn di_no_attr(name: &str, attr: &TokenStream) -> SynRes<()> {
    if attr.is_empty() {
        return Ok(());
    }
    let msg = format!("{name} should not have any argument");
    Err(SynErr::new(Span::call_site(), msg))
}
