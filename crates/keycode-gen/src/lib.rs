pub mod def;

use core::error::Error;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use regex::Regex;
use semver::Version;

use crate::def::{KeyRangeIns, KeycodeIns, Spec};

struct Item {
    version: Version,
    spec: Spec,
}

pub fn generate(dir: impl AsRef<Path>) -> Result<Spec, Box<dyn Error>> {
    let regex = Regex::new(r"keycodes_(\w+)\.(\w+)\.(\w)(?:_(.*))?\.hjson").unwrap();

    // Collect specs by categories
    let mut map = HashMap::<Option<String>, Vec<Item>>::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }

        let filename = entry.file_name();
        let Some(filename) = filename.to_str() else {
            continue;
        };
        let Some(captures) = regex.captures(filename) else {
            continue;
        };

        let major = captures.get(1).unwrap().as_str();
        let minor = captures.get(2).unwrap().as_str();
        let patch = captures.get(3).unwrap().as_str();
        let category = captures.get(4).map(|a| a.as_str().to_string());
        let version = Version::new(major.parse()?, minor.parse()?, patch.parse()?);
        if entry.metadata()?.len() == 0 {
            continue;
        }

        let spec = serde_hjson::from_reader::<_, Spec>(BufReader::new(File::open(entry.path())?))?;
        map.entry(category)
            .or_insert(vec![])
            .push(Item { version, spec });
    }

    // merge specs by category and version
    let mut merged = Spec::default();
    for (_, mut items) in map {
        items.sort_by(|a, b| a.version.cmp(&b.version));

        let mut category_merged = Spec::default();
        for item in items {
            merge_specs(&mut category_merged, item.spec);
        }

        merge_specs(&mut merged, category_merged);
    }

    Ok(merged)
}

fn merge_specs(dst: &mut Spec, src: Spec) {
    for (key, keycode_ins) in src.keycodes {
        match keycode_ins {
            KeycodeIns::Delete(_) => {
                dst.keycodes.remove(&key);
            }
            KeycodeIns::Reset(_) => {
                dst.keycodes.clear();
            }
            KeycodeIns::Def(_) => {
                dst.keycodes.insert(key, keycode_ins);
            }
        }
    }

    for (key, range_ins) in src.ranges {
        match range_ins {
            KeyRangeIns::Delete(_) => {
                dst.ranges.remove(&key);
            }
            KeyRangeIns::Def { .. } => {
                dst.ranges.insert(key, range_ins);
            }
        }
    }
}
