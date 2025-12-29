use std::fmt::Display;

use quote::{IdentFragment, ToTokens};
use syn::{Ident, Index};

pub enum Member {
    Named(Ident),
    Unnamed(Index),
}

impl Member {
    pub fn new(idx: usize, ident: Option<&Ident>) -> Self {
        ident.cloned().map(Self::Named).unwrap_or(idx.into())
    }
}

impl ToTokens for Member {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Named(ident) => ToTokens::to_tokens(ident, tokens),
            Self::Unnamed(idx) => ToTokens::to_tokens(idx, tokens),
        }
    }
}

impl From<usize> for Member {
    fn from(value: usize) -> Self {
        Self::Unnamed(value.into())
    }
}

impl IdentFragment for Member {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Named(ident) => IdentFragment::fmt(ident, f),
            Self::Unnamed(idx) => IdentFragment::fmt(idx, f),
        }
    }

    fn span(&self) -> Option<proc_macro2::Span> {
        match self {
            Self::Named(ident) => Some(ident.span()),
            Self::Unnamed(idx) => Some(idx.span),
        }
    }
}

impl Display for Member {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Named(ident) => write!(f, "{ident}"),
            Self::Unnamed(idx) => write!(f, "{}", idx.index),
        }
    }
}
