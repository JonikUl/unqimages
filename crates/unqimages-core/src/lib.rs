pub mod config;
pub mod discovery;
pub mod duplicates;
pub mod hash;
pub mod perceptual;
pub mod types;

pub use config::*;
pub use discovery::*;
pub use duplicates::*;
pub use hash::*;
pub use perceptual::*;
pub use types::*;

use std::io;

pub fn find_exact_duplicates(config: &Config) -> io::Result<Vec<DuplicateGroup>> {
    find_duplicates_impl(config, false)
}

pub fn find_duplicates(config: &Config) -> io::Result<Vec<DuplicateGroup>> {
    find_duplicates_impl(config, true)
}

fn find_duplicates_impl(
    config: &Config,
    include_perceptual: bool,
) -> io::Result<Vec<DuplicateGroup>> {
    // `find_exact_duplicates` must ignore perceptual settings even if the user
    // enabled them in the config, so the flag is passed explicitly.
    let mut entries = discover_images(config);

    for entry in &mut entries {
        entry.file_hash = Some(hash_file(&entry.path)?);
    }

    let perceptual = if include_perceptual {
        config.perceptual.as_ref()
    } else {
        None
    };

    if perceptual.map(|p| p.enabled).unwrap_or(false) {
        for entry in &mut entries {
            entry.perceptual_hash = compute_perceptual_hash(&entry.path)?;
        }
    }

    find_combined_duplicates(entries, perceptual)
}
