pub mod config;
pub mod discovery;
pub mod duplicates;
pub mod hash;
pub mod types;

pub use config::*;
pub use discovery::*;
pub use duplicates::*;
pub use hash::*;
pub use types::*;

use std::io;

pub fn find_exact_duplicates(config: &Config) -> io::Result<Vec<DuplicateGroup>> {
    let mut entries = discover_images(config);

    for entry in &mut entries {
        entry.file_hash = Some(hash_file(&entry.path)?);
    }

    Ok(group_by_file_hash(entries))
}
