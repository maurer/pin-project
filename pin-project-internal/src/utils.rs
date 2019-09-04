use quote::format_ident;
use syn::{
    punctuated::Punctuated,
    token::{self, Comma},
    Attribute, GenericParam, Generics, Ident, Lifetime, LifetimeDef,
};

pub(crate) const DEFAULT_LIFETIME_NAME: &str = "'_pin";
pub(crate) const TRAIT_LIFETIME_NAME: &str = "'_outer_pin";

/// Makes the ident of projected type from the reference of the original ident.
pub(crate) fn proj_ident(ident: &Ident) -> Ident {
    format_ident!("__{}Projection", ident)
}

pub(crate) fn proj_trait_ident(ident: &Ident) -> Ident {
    format_ident!("__{}ProjectionTrait", ident)
}

/// Determine the lifetime names. Ensure it doesn't overlap with any existing lifetime names.
pub(crate) fn proj_lifetime_name(
    lifetime_name: &mut String,
    generics: &Punctuated<GenericParam, Comma>,
) {
    let existing_lifetimes: Vec<String> = generics
        .iter()
        .filter_map(|param| {
            if let GenericParam::Lifetime(LifetimeDef { lifetime, .. }) = param {
                Some(lifetime.to_string())
            } else {
                None
            }
        })
        .collect();
    while existing_lifetimes.iter().any(|name| name.starts_with(&**lifetime_name)) {
        lifetime_name.push('_');
    }
}

/// Makes the generics of projected type from the reference of the original generics.
pub(crate) fn proj_generics(generics: &mut Generics, lifetime: Lifetime) {
    if let lt_token @ None = &mut generics.lt_token {
        *lt_token = Some(token::Lt::default())
    }
    if let gt_token @ None = &mut generics.gt_token {
        *gt_token = Some(token::Gt::default())
    }

    generics.params.insert(
        0,
        GenericParam::Lifetime(LifetimeDef {
            attrs: Vec::new(),
            lifetime,
            colon_token: None,
            bounds: Punctuated::new(),
        }),
    );
}

pub(crate) trait VecExt {
    fn find_remove(&mut self, ident: &str) -> Option<Attribute>;
}

impl VecExt for Vec<Attribute> {
    fn find_remove(&mut self, ident: &str) -> Option<Attribute> {
        self.iter().position(|attr| attr.path.is_ident(ident)).map(|i| self.remove(i))
    }
}

/// If the 'renamed' feature is enabled, we detect
/// the actual name of the 'pin-project' crate in the consumer's Cargo.toml
#[cfg(feature = "renamed")]
pub(crate) fn crate_path() -> Ident {
    // This is fairly subtle.
    // Normally, you would use `env!("CARGO_PKG_NAME")` to get the name of the package,
    // since it's set at compile time.
    // However, we're in a proc macro, which runs while *another* crate is being compiled.
    // By retreiving the runtime value of `CARGO_PKG_NAME`, we can figure out the name
    // of the crate that's calling us.
    let cur_crate = std::env::var("CARGO_PKG_NAME")
        .expect("Could not find CARGO_PKG_NAME environemnt variable");
    format_ident!(
        "{}",
        if cur_crate == "pin-project" { "pin_project" } else { crate::PIN_PROJECT_CRATE.as_str() },
    )
}

/// If the 'renamed' feature is not enabled, we just
/// assume that the 'pin-project' dependency has not been renamed
#[cfg(not(feature = "renamed"))]
pub(crate) fn crate_path() -> Ident {
    format_ident!("pin_project")
}

macro_rules! error {
    ($span:expr, $msg:expr) => {
        syn::Error::new_spanned(&$span, $msg)
    };
    ($span:expr, $($tt:tt)*) => {
        error!($span, format!($($tt)*))
    };
}
