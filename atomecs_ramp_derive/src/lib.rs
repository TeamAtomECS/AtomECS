extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;

/// Auto-generated implementation for Lerp trait.
///
/// Note that this macro assumes:
///  * The struct implements Clone.
///  * The struct only contains fields implementing Multiple and Add.
#[proc_macro_derive(Lerp)]
pub fn lerp_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_lerp_macro(&ast)
}

fn impl_lerp_macro(ast: &syn::DeriveInput) -> TokenStream {
    use syn::Fields;
    // let name = &ast.ident;
    // let mut tokens = quote!();

    // tokens.extend(quote!(
    // impl Lerp<#name> for #name {
    //     fn lerp(&self, other: &Self, amount: f64) -> Self {
    //         let mut output = self.clone();
    //         // assign field values for lerpable fields.
    // ));
    // // generate implementation for each field.
    // let fields = &data.fields;
    // match fields {
    // Fields::Named(named) => for field in named.named.iter() {
    //     let field_name = field.ident.unwrap();
    //     tokens.extent(quote!(
    //         output.#field_name = self.#field_name * (1.0 - amount) + amount * other.#field_name;
    //     ));
    // }
    // _ => unimplemented!()
    // }

    // tokens.extend(
    //     quote!(
    //             return output;
    //             }
    //         }
    //     ));
    // tokens.into()

    // let name = &ast.ident;
    // let data: &syn::Data = &ast.data;
    // match data {
    //     syn::Data::Struct(struct_data) => {
    //         use syn::Field;
    //         let fields: Vec<Field> = match &struct_data.fields {
    //             Fields::Named(named) => named.named.into_iter().collect(),
    //             _ => unimplemented!(),
    //         };
    //         let tokens = quote! {
    //             impl Lerp<#name> for #name {
    //             fn lerp(&self, other: &Self, amount: f64) -> Self {
    //                 let mut output = self.clone();
    //                 // assign field values for lerpable fields.
    //                 #(output.#fields = self.#fields * (1.0 - amount) + amount * other.#fields;)*
    //                 return output;
    //                 }
    //             }
    //         };
    //         tokens.into()
    //     }

    //     _ => unimplemented!(),
    // }

    let name = &ast.ident;
    let data: &syn::Data = &ast.data;
    match data {
        syn::Data::Struct(struct_data) => {
            let mut field_assignment_tokens = quote!();
            match &struct_data.fields {
                Fields::Named(named) => {
                    for field in named.named.iter() {
                        let field_name = field.ident.as_ref();
                        field_assignment_tokens.extend(
                            quote!(output.#field_name = self.#field_name * (1.0 - amount) + amount * other.#field_name;)
                            );
                    }
                }
                _ => unimplemented!(),
            }

            let tokens = quote! {
                impl Lerp<#name> for #name {
                fn lerp(&self, other: &Self, amount: f64) -> Self {
                    let mut output = self.clone();
                    // assign field values for lerpable fields.
                    #field_assignment_tokens
                    return output;
                    }
                }
            };
            tokens.into()
        }

        _ => unimplemented!(),
    }
}
