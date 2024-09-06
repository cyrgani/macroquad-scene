use std::iter::Peekable;

use proc_macro::TokenTree;

fn next_group(source: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Option<proc_macro::Group> {
    if let Some(TokenTree::Group(_)) = source.peek() {
        let group = match source.next().unwrap() {
            TokenTree::Group(group) => group,
            _ => unreachable!("just checked with peek()!"),
        };
        Some(group)
    } else {
        None
    }
}

/// Maybe this will go away in future versions.
#[doc(hidden)]
#[proc_macro_derive(CapabilityTrait)]
pub fn capability_trait_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut source = input.into_iter().peekable();

    while let Some(TokenTree::Punct(_)) = source.peek() {
        let _ = source.next();
        let _ = next_group(&mut source);
    }
    assert_eq!("pub", &format!("{}", source.next().unwrap()));
    assert_eq!("struct", &format!("{}", source.next().unwrap()));
    let struct_name = format!("{}", source.next().unwrap());

    let mut group = next_group(&mut source)
        .unwrap()
        .stream()
        .into_iter()
        .peekable();

    let mut trait_decl = format!("pub trait {}Trait {{", struct_name);
    let mut trait_impl = format!("impl {}Trait for NodeWith<{}> {{", struct_name, struct_name);

    fn next_str(group: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Option<String> {
        group.next().map(|tok| format!("{}", tok))
    }

    loop {
        // skip doc comments
        while let Some(TokenTree::Punct(_)) = group.peek() {
            let _ = group.next();
            let _ = next_group(&mut group);
        }

        let _pub = next_str(&mut group);
        if _pub.is_none() {
            break;
        }
        assert_eq!("pub", _pub.unwrap());
        let fn_name = next_str(&mut group).unwrap();
        let mut fn_res = "()".to_string();
        assert_eq!(":", &next_str(&mut group).unwrap());
        assert_eq!("fn", &next_str(&mut group).unwrap());
        let fn_args_decl = next_str(&mut group).unwrap();
        let mut fn_args_impl = String::new();

        let args = fn_args_decl.split(":").collect::<Vec<&str>>();
        for arg in &args[1..args.len() - 1] {
            fn_args_impl.push_str(&format!(", {}", arg.split(", ").last().unwrap()));
        }
        let p = next_str(&mut group);
        match p.as_deref() {
            Some("-") => {
                assert_eq!(">", next_str(&mut group).unwrap());
                fn_res = next_str(&mut group).unwrap();
                let _ = next_str(&mut group);
            }
            Some(",") => {}
            None => break,
            _ => panic!(),
        };

        trait_decl.push_str(&format!(
            "fn {} {} -> {};",
            fn_name,
            fn_args_decl
                .replace("node : HandleUntyped", "&self")
                .replace("node: HandleUntyped", "&self"),
            fn_res
        ));

        let args = fn_args_impl
            .replace("node : HandleUntyped", "")
            .replace("node: HandleUntyped", "")
            .replace("(", "")
            .replace(")", "");

        trait_impl.push_str(&format!(
            "fn {} {} -> {} {{",
            fn_name,
            fn_args_decl
                .replace("node : HandleUntyped", "&self")
                .replace("node: HandleUntyped", "&self"),
            fn_res
        ));
        trait_impl.push_str(&format!(
            "(self.capability.{})(self.node {})",
            fn_name, args
        ));
        trait_impl.push_str("}");
    }
    trait_decl.push_str("}");
    trait_impl.push_str("}");

    let res = format!(
        "{} 
{}",
        trait_decl, trait_impl
    );
    res.parse().unwrap()
}
