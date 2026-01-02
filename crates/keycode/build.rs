use core::error::Error;
use std::{
    env, fs,
    io::{BufWriter, Write},
    path::PathBuf,
};

use keycode_gen::def::{KeyRange, KeyRangeIns, KeycodeIns};
use quote::{format_ident, quote};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=qmk");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");

    let spec = keycode_gen::generate("./qmk/data/constants/keycodes")?;

    let keycodes = spec.keycodes.into_iter().filter_map(|(key, value)| {
        let KeycodeIns::Def(def) = value else {
            return None;
        };

        let name = format_ident!("{}", def.key);
        let keycode = u16::from_str_radix(&key[2..], 16).expect("invalid keycode");
        let desc = def.label.and_then(|label| {
            if label.is_empty() {
                None
            } else {
                Some(quote!(#[doc = #label]))
            }
        });

        Some(quote! {
            #desc
            pub const #name: Self = Self(#keycode);
        })
    });
    let ranges = spec.ranges.into_iter().filter_map(|(key, value)| {
        let KeyRangeIns::Def { define } = value else {
            return None;
        };

        let start_name = format_ident!("RANGE_{}_START", define);
        let end_name = format_ident!("RANGE_{}_END", define);
        let KeyRange { start, size } = key;
        let end = start + size;
        Some(quote!(
            pub const #start_name: u16 = #start;
            pub const #end_name: u16 = #end;
        ))
    });

    let tokens = quote! {
        impl Keycode {
            #(#ranges)*

            #(#keycodes)*
        }
    };

    let file = fs::File::create(PathBuf::from(&out_dir).join("impl_keycode.rs"))?;
    let mut writer = BufWriter::new(file);
    write!(writer, "{}", prettyplease::unparse(&syn::parse2(tokens)?))?;

    Ok(())
}
