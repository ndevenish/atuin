use std::path::PathBuf;

use super::Importer;
use crate::import::read_to_end;
use async_trait::async_trait;
use directories::UserDirs;
use eyre::{Result, eyre};

pub struct FullHistory {
    bytes: Vec<u8>,
}

fn fullhistory_db_path() -> Result<PathBuf> {
    let home_dir = UserDirs::new()
        .ok_or_else(|| eyre!("Could not find user directories"))?
        .home_dir()
        .join(".fullhistory");
    Ok(home_dir)
}

#[async_trait]
impl Importer for FullHistory {
    const NAME: &'static str = "fullhistory";

    async fn new() -> Result<Self> {
        let bytes = read_to_end(fullhistory_db_path()?)?;
        Ok(Self { bytes })
    }

    async fn entries(&mut self) -> Result<usize> {
        Ok(super::count_lines(&self.bytes))
    }

    async fn load(self, h: &mut impl Loader) -> Result<()> {}
}
