use core::panic;

use heck::ToUpperCamelCase;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use syn::{
    parse2, parse_quote,
    visit::{visit_item_fn, Visit},
    GenericArgument, ItemFn, Pat, PatIdent, PathArguments, ReturnType, Type,
};

#[derive(Default)]
struct FunctionVisitor {
    fn_ident: Option<Ident>,

    ident: Option<String>,
    args: Vec<(PatIdent, Type)>,
    state: Option<Type>,
    ctx_path_args: Option<PathArguments>,
}

impl FunctionVisitor {}

impl Visit<'_> for FunctionVisitor {
    fn visit_item_fn(&mut self, func: &'_ ItemFn) {
        visit_item_fn(self, func);

        if func.sig.variadic.is_some() {
            panic!("functional widgets do not support variadic arguments");
        }

        if let ReturnType::Default = func.sig.output {
            panic!("return type must be BuildResult");
        }

        if let ReturnType::Type(_, ty) = &func.sig.output {
            if !matches!(**ty, Type::Path(_)) {
                panic!("return type must be BuildResult");
            }
        }

        self.fn_ident = Some(func.sig.ident.clone());
        self.ident = Some(func.sig.ident.to_string().to_upper_camel_case());

        for input in &func.sig.inputs {
            match input {
                syn::FnArg::Receiver(_) => {
                    panic!("functional widgets do not support self");
                }
                syn::FnArg::Typed(arg) => {
                    let pat = &*arg.pat;
                    let ty = &*arg.ty;

                    if let Pat::Ident(ident) = pat {
                        self.args.push((ident.clone(), ty.clone()));
                    } else {
                        panic!("unexpected argument: {:?}", pat);
                    }
                }
            }
        }

        if !self.args.is_empty() {
            let (_, ty) = self.args.remove(0);

            if let Type::Reference(ty) = ty {
                if ty.mutability.is_none() {
                    panic!("first argument must be &mut BuildContext");
                }

                if let Type::Path(ty_path) = &*ty.elem {
                    let segment = ty_path.path.segments.last().unwrap();

                    if segment.ident != "BuildContext" {
                        panic!("first argument must be &mut BuildContext");
                    }

                    self.ctx_path_args = Some(segment.arguments.clone());

                    if let PathArguments::AngleBracketed(generic) = &segment.arguments {
                        if let GenericArgument::Type(ty) = generic.args.first().unwrap() {
                            self.state = Some(ty.clone());
                        }
                    }
                } else {
                    panic!("first argument must be &mut BuildContext");
                }
            }
        } else {
            panic!("first argument must be &mut BuildContext");
        }
    }
}

pub(crate) fn parse_functional_widget(_args: TokenStream2, item: TokenStream2) -> TokenStream2 {
    let item = match parse2(item) {
        Ok(item) => item,
        Err(err) => return err.into_compile_error(),
    };

    let mut visitor = FunctionVisitor::default();

    visitor.visit_item_fn(&item);

    let fn_ident = visitor
        .fn_ident
        .expect("functional widget formatted incorrectly");

    let ident = Ident::new(
        &visitor
            .ident
            .expect("functional widget formatted incorrectly"),
        Span::call_site(),
    );

    let mut fields = quote::quote! {};
    let mut args = quote::quote! { ctx };

    for (ident, ty) in &visitor.args {
        fields.extend(quote::quote! {
            pub #ident: #ty,
        });

        args.extend(quote::quote! {
            , self.#ident.clone()
        });
    }

    let state = visitor.state;
    let ctx_path_args = match visitor.ctx_path_args {
        Some(args) => quote::quote! { #args },
        None => quote::quote! {},
    };

    // #[cfg(feature = "internal")]
    // let agui_core = quote::quote! { agui_core };
    // #[cfg(not(feature = "internal"))]
    let agui_core = quote::quote! { agui };

    parse_quote! {
        #item

        #[derive(Debug, Default)]
        pub struct #ident {
            #fields
        }

        impl #agui_core::widget::StatefulWidget for #ident {
            type State = #state;

            fn build(&self, ctx: &mut #agui_core::widget::BuildContext #ctx_path_args) -> #agui_core::widget::BuildResult {
                #fn_ident(#args)
            }
        }
    }
}
