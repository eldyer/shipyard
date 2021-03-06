extern crate proc_macro;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_quote, Error, Result};

const MAX_TYPES: usize = 10;

#[allow(clippy::or_fun_call)]
#[proc_macro_attribute]
pub fn system(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let name = syn::parse_macro_input!(attr as syn::Ident);
    let run = syn::parse_macro_input!(item as syn::ItemFn);
    expand_system(name, run)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn expand_system(name: syn::Ident, mut run: syn::ItemFn) -> Result<TokenStream> {
    if run.sig.ident != "run" {
        return Err(Error::new(
            Span::call_site(),
            "Systems have only one method: run",
        ));
    }
    if !run.sig.generics.params.is_empty() {
        return Err(Error::new_spanned(
            run.sig.generics,
            "run should not take generic arguments",
        ));
    }
    if run.sig.generics.where_clause.is_some() {
        return Err(Error::new_spanned(
            run.sig.generics.where_clause,
            "run should not take a where clause",
        ));
    }

    // checks if run returns a type other than ()
    let returns_something = match run.sig.output {
        syn::ReturnType::Type(_, ref type_info) => match **type_info {
            syn::Type::Tuple(ref tuple) => !tuple.elems.is_empty(),
            _ => true,
        },
        syn::ReturnType::Default => false,
    };
    if returns_something {
        return Err(Error::new_spanned(
            run.sig.output,
            "run should not return anything",
        ));
    }

    let body = &*run.block;
    let vis = run.vis;

    let mut data = Vec::with_capacity(run.sig.inputs.len());
    let mut binding = Vec::with_capacity(run.sig.inputs.len());

    run.sig.inputs.iter_mut().try_for_each(|arg| {
        if let syn::FnArg::Typed(syn::PatType { pat, ty, .. }) = arg {
            match **ty {
                syn::Type::Reference(ref mut reference) => {
                    // references are added a 'a lifetime if they don't have one
                    if let syn::Type::Path(path) = &*reference.elem {
                        // transform &Entities into Entites and &mut Entities into EntitiesMut
                        if path.path.segments.last().unwrap().ident == "Entities" {
                            if reference.mutability.is_none() {
                                **ty = parse_quote!(::shipyard::prelude::Entities);
                            } else {
                                **ty = parse_quote!(::shipyard::prelude::EntitiesMut);
                            }
                        } else if reference.lifetime.is_none() {
                            reference.lifetime = parse_quote!('a);
                        }
                    } else if reference.lifetime.is_none() {
                        reference.lifetime = parse_quote!('a);
                    }
                }
                syn::Type::Path(ref mut path) => {
                    let last = path.path.segments.last_mut().unwrap();
                    // Not has to be handled separately since its lifetime is inside the type
                    if last.ident == "Not" {
                        if let syn::PathArguments::AngleBracketed(inner_type) = &mut last.arguments
                        {
                            if inner_type.args.len() != 1 {
                                return Err(Error::new_spanned(
                                    last,
                                    "Not will only accept one type and nothing else",
                                ));
                            }
                            let arg = inner_type.args.iter_mut().next().unwrap();
                            if let syn::GenericArgument::Type(inner_type) = arg {
                                if let syn::Type::Reference(reference) = inner_type {
                                    if reference.lifetime.is_none() {
                                        reference.lifetime = parse_quote!('a);
                                    }
                                } else {
                                    return Err(Error::new_spanned(
                                        inner_type,
                                        "Not will only work with component storages refered by &T or &mut T",
                                    ));
                                }
                            } else {
                                unreachable!()
                            }
                        }
                    }
                }
                _ => {
                    return Err(Error::new_spanned(
                        ty,
                        "A system will only accept a type of this list:\n\
                            \t\t- &T for an immutable reference to T storage\n\
                            \t\t- &mut T for a mutable reference to T storage\n\
                            \t\t- &Entities for an immutable reference to the entity storage\n\
                            \t\t- &mut EntitiesMut for a mutable reference to the entity storage\n\
                            \t\t- AllStorages for a mutable reference to the storage of all components\n\
                            \t\t- ThreadPool for an immutable reference to the rayon::ThreadPool used by the World",
                    ));
                }
            }

            data.push((*ty).clone());
            binding.push((**pat).clone());
            Ok(())
        } else {
            unreachable!()
        }
    })?;

    // make tuples MAX_TYPES len maximum to allow users to pass an infinite number of types
    while data.len() > MAX_TYPES {
        for i in 0..(data.len() / MAX_TYPES) {
            let ten = &data[(i * MAX_TYPES)..((i + 1) * MAX_TYPES)];
            *data[i] = parse_quote!((#(#ten,)*));
            data.drain((i + 1)..((i + 1) * MAX_TYPES));

            let ten = &binding[i..(i + 10)];
            binding[i] = parse_quote!((#(#ten,)*));
            binding.drain((i + 1)..((i + 1) * MAX_TYPES));
        }
    }

    Ok(quote! {
        #vis struct #name;
        impl<'a> ::shipyard::prelude::System<'a> for #name {
            type Data = (#(#data,)*);
            fn run((#(#binding,)*): <Self::Data as ::shipyard::prelude::SystemData<'a>>::View) #body
        }
    })
}
