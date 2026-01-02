pub mod def;

use core::error::Error;
use std::{
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use regex::Regex;
use semver::Version;

use crate::def::Spec;

struct Item {
    version: Version,
    spec: Spec,
}

pub fn generate(dir: impl AsRef<Path>) -> Result<Spec, Box<dyn Error>> {
    let regex = Regex::new(r"keycodes_(\w+)\.(\w+)\.(\w).*\.hjson").unwrap();

    let mut items = vec![];
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

        let (_, [major, minor, patch]) = captures.extract::<3>();
        let version = Version::new(major.parse()?, minor.parse()?, patch.parse()?);
        if entry.metadata()?.len() == 0 {
            continue;
        }

        let spec = serde_hjson::from_reader::<_, Spec>(BufReader::new(File::open(entry.path())?))?;
        items.push(Item { version, spec });
    }

    items.sort_by(|a, b| a.version.cmp(&b.version));
    let mut merged = Spec::default();
    for item in items {
        merged.keycodes.extend(item.spec.keycodes);
        merged.ranges.extend(item.spec.ranges);
    }

    Ok(merged)
}
