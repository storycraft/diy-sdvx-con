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
        let desc = def.label.unwrap_or_default();
        Some(quote! {
            #[doc = #desc]
            pub const #name: Self = Self(#keycode);
        })
    });
    let ranges = spec.ranges.into_iter().filter_map(|(key, value)| {
        let KeyRangeIns::Def { define } = value else {
            return None;
        };
        
        let name = format_ident!("RANGE_{}", define);
        let KeyRange { start, end } = key;
        Some(quote! {
            pub const #name: ::core::ops::Range<u16> = #start..#end;
        })
    });

    let tokens = quote! {
        #[derive(Clone, Copy, Hash, PartialEq, Eq, zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable)]
        #[repr(transparent)]
        pub struct Keycode(pub u16);

        impl Keycode {
            #(#ranges)*

            #(#keycodes)*
        }
    };

    let file = fs::File::create(PathBuf::from(&out_dir).join("keycode.rs"))?;
    let mut writer = BufWriter::new(file);
    for token in tokens {
        write!(writer, "{token} ")?;
    }

    Ok(())
}
