// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::iter::Peekable;

#[cfg(not(kernel))]
use proc_macro2 as proc_macro;

use proc_macro::Punct;
use proc_macro::{Delimiter, Ident, Spacing, TokenStream, TokenTree};

pub fn expand(input: TokenStream) -> TokenStream {
    let mut tokens = input.into_iter().peekable();
    let attrs = parse_attrs(&mut tokens);
    let default_error = attrs
        .iter()
        .filter_map(|attr| match attr {
            Attr::Pin => None,
            Attr::DefaultError(err) => Some(err),
        })
        .next_back();
    let closure = parse_closure(&mut tokens, default_error);
    let mut statements = vec![];
    let tail = match tokens.peek() {
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
            tokens.next();
            match tokens.next() {
                None => {}
                Some(rest) => panic!("unexpected token after initializer body: {rest:?}"),
            }
            loop {
                let mut statement: TokenStream = (&mut tokens)
                    .take_while(|t| !matches!(t, TokenTree::Punct(p) if p.as_char() == ';'))
                    .collect();
                match tokens.peek() {
                    None => break parse_initializer_tail(statement),
                    Some(TokenTree::Punct(p)) if p.as_char() == ';' => {
                        statement.extend([tokens.next().unwrap()])
                    }
                    Some(_) => unreachable!(),
                }
                statements.push(statement);
            }
        }
        Some(_) => {
            if closure.is_some() {
                panic!("expected initializer body when using closure")
            }
            parse_initializer_tail(&mut tokens)
        }
        None => panic!("missing initializer body"),
    };
    let Tail { path, fields } = tail;
    let ty = closure
        .as_ref()
        .and_then(|c| c.ty.as_ref().cloned())
        .unwrap_or_else(|| quote!(_));
    let err = closure
        .as_ref()
        .and_then(|c| c.err.as_ref().cloned())
        .or(default_error.cloned())
        .unwrap_or_else(|| quote!(::core::convert::Infallible));
    quote! {
        ::pin_init::__init_internal!(
            statements(#(#statements)*),
            ty_hint(#ty),
            err(#err),
            struct_path(#(#path)*),
            fields(#fields),
        )
    }
}

struct Tail {
    path: Vec<TokenTree>,
    fields: TokenStream,
}

fn parse_initializer_tail(tokens: impl IntoIterator<Item = TokenTree>) -> Tail {
    let mut tokens: Vec<TokenTree> = tokens.into_iter().collect();
    if tokens.is_empty() {
        panic!("incomplete initializer body")
    }
    let last = tokens.remove(tokens.len() - 1);
    match last {
        TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => Tail {
            path: tokens,
            fields: g.stream(),
        },
        _ => panic!("expected `{{}}` as the last token in the initializer body, found {last:?}"),
    }
}

enum Attr {
    Pin,
    DefaultError(TokenStream),
}

fn parse_attr(meta: TokenStream) -> Attr {
    let mut tokens = meta.into_iter();
    match tokens.next() {
        Some(TokenTree::Ident(name)) => {
            if name == "pin" {
                match tokens.next() {
                    None => {}
                    Some(next) => panic!("unexpected token in `#[pin]` attribute: {next:?}"),
                }
                Attr::Pin
            } else if name == "default_error" {
                Attr::DefaultError(tokens.collect())
            } else {
                panic!("unexpected attribute name: `{name}`")
            }
        }
        Some(rest) => panic!("unexpected token in attribute: {rest:?}"),
        None => panic!("expected name inside of attribute"),
    }
}

fn parse_attrs(tokens: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Vec<Attr> {
    let mut attrs = vec![];
    loop {
        match tokens.peek() {
            Some(TokenTree::Punct(p)) if p.as_char() == '#' => {
                tokens.next();
                match tokens.next() {
                    Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Bracket => {
                        attrs.push(parse_attr(g.stream()));
                    }
                    next => {
                        panic!("expected `[...]` after `#`, but found {next:?}")
                    }
                }
            }
            Some(_) => break attrs,
            None => panic!("missing initializer body"),
        }
    }
}

enum Arg {
    Untyped(Ident),
    Typed { name: Ident, ty: TokenStream },
}

struct ClosureSig {
    /// arguments in the `|arg0, arg1|`
    args: Vec<Arg>,
    /// returned type `-> MyType` or `-> Result<MyType, _>`
    ty: Option<TokenStream>,
    /// returned error type `-> Result<, Err>`
    err: Option<TokenStream>,
}

fn parse_closure(
    tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
    default_error: Option<&TokenStream>,
) -> Option<ClosureSig> {
    match tokens.peek() {
        Some(TokenTree::Punct(p)) if p.as_char() == '|' => {
            tokens.next();
        }
        Some(_) => return None,
        None => panic!("missing initializer body"),
    }
    let mut args = vec![];
    loop {
        match tokens.next() {
            Some(TokenTree::Ident(name)) => {
                match tokens.peek() {
                    Some(TokenTree::Punct(p)) if p.as_char() == ':' => {
                        tokens.next();
                        args.push(Arg::Typed {
                            name: name.clone(),
                            ty: parse_ty_until_punct(tokens, |p| matches!(p.as_char(), ',' | '|')),
                        });
                    }
                    _ => args.push(Arg::Untyped(name.clone())),
                }
                match tokens.peek() {
                    Some(TokenTree::Punct(p)) if p.as_char() == ',' => {
                        tokens.next();
                    }
                    Some(TokenTree::Punct(p)) if p.as_char() == '|' => break,
                    Some(rest) => {
                        panic!("expected comma after argument in initializer closure signature: {rest:?}")
                    }
                    _ => {}
                }
            }
            Some(TokenTree::Punct(p)) if p.as_char() == '|' => break,
            Some(rest) => panic!("unexpected token in initializer closure signature: {rest:?}"),
            None => panic!("incomplete initializer body"),
        }
    }
    // check for an `->` indicating a return type
    match tokens.peek() {
        Some(TokenTree::Punct(p)) if p.as_char() == '-' && p.spacing() == Spacing::Joint => {
            tokens.next();
            match tokens.next() {
                Some(TokenTree::Punct(p)) if p.as_char() == '>' => {}
                Some(rest) => panic!("expected arrow `->` in initializer closure signature, found `-` and then {rest:?}"),
                None => panic!("incomplete initializer body"),
            }
        }
        _ => {
            return Some(ClosureSig {
                args,
                ty: None,
                err: None,
            })
        }
    }
    // we support several different constructs here before the opening `{`:
    // * just having your own type here (then the error will be assumed to be `Infallible`),
    // * you can have `Result<InitializedType, Error>`,
    // * `Result<InitializedType, _>` combined with a `#[default_error(Err)]` attribute,
    match tokens.peek() {
        Some(TokenTree::Ident(res)) if res == "Result" => {
            tokens.next();
            match tokens.next() {
                Some(TokenTree::Punct(p)) if p.as_char() == '<' => {}
                _ => panic!("expected `<` after `Result` in initializer return type"),
            }
            let ty = parse_ty_until_punct(tokens, |p| p.as_char() == ',');
            match tokens.next() {
                Some(TokenTree::Punct(p)) if p.as_char() == ',' => {}
                _ => {
                    panic!("expected `,` after first type in `Result<` in initializer return type")
                }
            }
            let mut err = parse_ty_until_punct(tokens, |p| matches!(p.as_char(), ',' | '>'));
            match tokens.next() {
                Some(TokenTree::Punct(p)) if p.as_char() == ',' => match tokens.next() {
                    Some(TokenTree::Punct(p)) if p.as_char() == '>' => {}
                    _ => {
                        panic!("expected `>` after second type in `Result<` in initializer return type")
                    }
                },
                Some(TokenTree::Punct(p)) if p.as_char() == '>' => {}
                _ => {
                    panic!("expected `,` or `>` after second type in `Result<` in initializer return type")
                }
            }
            let mut err_inspect = err.into_iter().peekable();
            if matches!(&err_inspect.peek(), Some(TokenTree::Punct(p)) if p.as_char() == '_') {
                assert_eq!(
                    err_inspect.count(),
                    1,
                    "expected type to only be `_` if it starts with `_`"
                );
                err = default_error
                    .expect(
                        "need a `#[default_error()]` attribute to be able to use `_` in errors.",
                    )
                    .clone();
            } else {
                err = err_inspect.collect();
            }
            Some(ClosureSig {
                args,
                ty: Some(ty),
                err: Some(err),
            })
        }
        None => panic!("incomplete initializer body"),
        _ => Some(ClosureSig {
            args,
            ty: Some(parse_ty_until_brace(tokens)),
            err: None,
        }),
    }
}

fn parse_ty_until_punct(
    tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
    mut punct: impl FnMut(&Punct) -> bool,
) -> TokenStream {
    let mut nesting = 0u64;
    let mut res = TokenStream::new();
    loop {
        match tokens.peek() {
            Some(TokenTree::Punct(p)) => {
                if nesting == 0 && punct(p) {
                    return res;
                }
                match p.as_char() {
                    '<' => nesting += 1,
                    '>' => {
                        nesting = nesting
                            .checked_sub(1)
                            .expect("nestings of `<`/`>` became negative");
                    }
                    _ => {}
                }
            }
            Some(_) => {}
            None => panic!("incomplete initializer body"),
        }
        let Some(tok) = tokens.next() else {
            unreachable!()
        };
        res.extend([tok]);
    }
}

fn parse_ty_until_brace(tokens: &mut Peekable<impl Iterator<Item = TokenTree>>) -> TokenStream {
    let mut nesting = 0u64;
    let mut res = TokenStream::new();
    loop {
        match tokens.peek() {
            Some(TokenTree::Punct(p)) => match p.as_char() {
                '<' => nesting += 1,
                '>' => {
                    nesting = nesting
                        .checked_sub(1)
                        .expect("nestings of `<`/`>` became negative");
                }
                _ => {}
            },
            Some(TokenTree::Group(g)) if nesting == 0 && g.delimiter() == Delimiter::Brace => {
                return res;
            }
            Some(_) => {}
            None => panic!("incomplete initializer body"),
        }
        let Some(tok) = tokens.next() else {
            unreachable!()
        };
        res.extend([tok]);
    }
}
