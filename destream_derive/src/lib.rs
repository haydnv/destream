use proc_macro::TokenStream;
use syn::Ident;

#[proc_macro_derive(FromStream)]
pub fn from_stream_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_from_stream(&ast)
}

fn impl_from_stream(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let fields = get_field_names(&ast);
    let field_names: Vec<syn::LitStr> = fields
        .iter()
        .map(|f| {
            let original = f.to_string();
            let transformed = original.replace('_', "");
            syn::LitStr::new(&transformed, f.span())
        })
        .collect();

    let gen = quote::quote! {
        #[async_trait::async_trait]
        impl destream::FromStream for #name {
            type Context = ();

            async fn from_stream<D: destream::Decoder>(_context: Self::Context, decoder: &mut D) -> Result<Self, D::Error> {
                struct Visitor;

                #[async_trait::async_trait]
                impl destream::Visitor for Visitor {
                    type Value = #name;

                    fn expecting() -> &'static str {
                        stringify!(#name)
                    }

                    async fn visit_map<M: destream::MapAccess>(self, mut map: M) -> Result<Self::Value, M::Error>
                    {
                        // for each field in the ast
                        #(
                            let mut #fields = None;
                        )*
                        while let Some(key) = map.next_key::<String>(()).await? {
                            match key.as_str() {
                                #(
                                    #field_names => #fields = map.next_value(()).await?,
                                )*
                                _ => unimplemented!()
                            }

                        }

                        Ok(#name {
                            #(#fields,)*
                        })
                    }
                }

                let visitor = Visitor;
                decoder.decode_map(visitor).await
            }
        }
    };
    gen.into()
}

fn get_field_names(ast: &syn::DeriveInput) -> Vec<&Ident> {
    let fields = match &ast.data {
        syn::Data::Struct(data_struct) => match &data_struct.fields {
            syn::Fields::Named(fields_named) => &fields_named.named,
            syn::Fields::Unnamed(_) => {
                unimplemented!("Unnamed structs are not supported by derive(FromStream)")
            }
            syn::Fields::Unit => {
                unimplemented!("Unit structs are not supported by derive(FromStream)");
            }
        },
        _ => unimplemented!("derive(FromStream) only works on structs"),
    };

    fields
        .iter()
        .map(|f| f.ident.as_ref().expect("expected named fields"))
        .collect()
}
