use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input,
    DeriveInput,
};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);

    dbg!(&parsed);

    let name = &parsed.ident;
    let builder_name = format!("{}Builder", name);
    let builder_ident = syn::Ident::new(&builder_name, name.span());

    dbg!(&builder_name);

    let struct_builder = quote! {
        pub struct #builder_ident {
          executable: Option<String>,
          args: Option<Vec<String>>,
          env: Option<Vec<String>>,
          current_dir: Option<String>,
        }

        impl #builder_ident {
            pub fn executable(&mut self, value: String) -> &mut Self {
                self.executable = Some(value);
                self
            }

            pub fn args(&mut self, value: Vec<String>) -> &mut Self {
                self.args = Some(value);
                self
            }

            pub fn env(&mut self, value: Vec<String>) -> &mut Self {
                self.env = Some(value);
                self
            }

            pub fn current_dir(&mut self, value: String) -> &mut Self {
                self.current_dir = Some(value);
                self
            }

            pub fn build(&self) -> Result<Command, Box<dyn std::error::Error>> {
                Ok(#name {
                    executable: self.executable.clone().ok_or("missing executable")?,
                    args: self.args.clone().ok_or("missing args")?,
                    env: self.env.clone().ok_or("missing env")?,
                    current_dir: self.current_dir.clone().ok_or("missing current_dir")?,
                })
            }
        }
    };

    let struct_impl = quote! {
        impl #name {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }
    };

    let tokens = quote! {
        #struct_builder
        #struct_impl
    };

    TokenStream::from(tokens)
}
