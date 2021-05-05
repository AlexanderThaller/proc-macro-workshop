use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input,
    DeriveInput,
};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);

    let name = &parsed.ident;
    let builder_name = format!("{}Builder", name);
    let builder_ident = syn::Ident::new(&builder_name, name.span());

    let fields = match parsed.data {
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Named(named) => named.named,
            syn::Fields::Unnamed(_) => unimplemented!(),
            syn::Fields::Unit => unimplemented!(),
        },
        syn::Data::Enum(_) => unimplemented!(),
        syn::Data::Union(_) => unimplemented!(),
    };

    let check_type_is_option = |ty: &syn::Type| -> Option<_> {
        if let syn::Type::Path(path) = ty {
            let segment = &path.path.segments[0];
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(inner_type) = &segment.arguments {
                    let inner_ident = &inner_type.args;
                    return Some(inner_ident.clone());
                }
            }
        }

        None
    };

    let fields_ident = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;

        if let Some(inner_ident) = check_type_is_option(ty) {
            quote! { #name: std::option::Option<#inner_ident> }
        } else {
            quote! { #name: std::option::Option<#ty> }
        }
    });

    let setters = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        let setter_documentation = format!("Set value of field {}", name.as_ref().unwrap());

        if let Some(inner_ident) = check_type_is_option(ty) {
            return quote! {
                #[doc = #setter_documentation]
                pub fn #name(&mut self, value: #inner_ident) -> &mut Self {
                    self.#name = Some(value);
                    self
                }
            };
        } else {
            quote! {
                #[doc = #setter_documentation]
                pub fn #name(&mut self, value: #ty) -> &mut Self {
                    self.#name = Some(value);
                    self
                }
            }
        }
    });

    let build_fn = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;

        if check_type_is_option(ty).is_some() {
            return quote! {
                #name: self.#name.clone()
            };
        } else {
            quote! {
                #name: self.#name.clone().ok_or(concat!(stringify!(#name), " is missing"))?
            }
        }
    });

    let builder_fields = fields.iter().map(|f| {
        let name = &f.ident;

        quote! {
            #name: None
        }
    });

    let builder_documentation = format!("Builder for {}", name);
    let build_documentation = format!(
        "Build a new {} from the values set in {builder_name}. Panics if any field in \
         {builder_name} is unset.",
        name,
        builder_name = builder_name
    );

    let struct_builder = quote! {
        #[doc = #builder_documentation]
        pub struct #builder_ident {
          #(#fields_ident),*
        }

        impl #builder_ident {
            #(#setters)*

            #[doc = #build_documentation]
            pub fn build(&self) -> Result<Command, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fn),*
                })
            }
        }
    };

    let builder_new_documentation = format!("Create a new empty {}", builder_name);

    let struct_impl = quote! {
        impl #name {
            #[doc = #builder_new_documentation]
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#builder_fields),*
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
