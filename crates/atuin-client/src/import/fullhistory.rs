use std::path::PathBuf;

use super::{Importer, Loader};
use crate::{history::History, import::read_to_end};
use async_trait::async_trait;
use directories::UserDirs;
use eyre::{Result, eyre};
// use regex::bytes::Regex;
use fancy_regex::Regex;
use time::OffsetDateTime;

pub struct FullHistory {
    data: String,
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
        Ok(Self {
            data: String::from_utf8(bytes).unwrap(),
        })
    }

    async fn entries(&mut self) -> Result<usize> {
        let counter =
            Regex::new(r#"(?m)^[a-zA-Z0-9.-]+:"[^"]*" \d+ .*(?!^[a-zA-Z0-9.-]+:"[^"]*" \d+ )"#)
                .unwrap();
        let count = counter.find_iter(&self.data).count();
        println!("Count: {count}");
        Ok(count)
    }

    async fn load(self, h: &mut impl Loader) -> Result<()> {
        let parser = Regex::new(
            r#"(?m)^([a-zA-Z0-9.-]+):"([^"]*)" (\d+) ([^ ]+) (.*)(?!^[a-zA-Z0-9.-]+:"[^"]*" \d+ )"#,
        )
        .unwrap();
        for entry in parser.captures_iter(&self.data).map(|x| x.unwrap()) {
            let hostname = entry.get(1).unwrap().as_str();
            let cwd = entry.get(2).unwrap().as_str();
            let session = entry.get(3).unwrap().as_str();
            let timestamp = entry.get(4).unwrap().as_str();
            let command = entry.get(5).unwrap().as_str();

            let Ok(time) = OffsetDateTime::parse(
                &timestamp,
                &time::format_description::well_known::Iso8601::DEFAULT,
            ) else {
                panic!("Could not parse '{}' as ISO8601", timestamp);
            };

            // let x = entry;
            let hist: History = History::import()
                .command(command.trim())
                .cwd(cwd)
                .hostname(hostname)
                .session(session)
                .timestamp(time)
                .build()
                .into();
            // println!("{hist:?}");
            h.push(hist).await.unwrap();
        }
        Ok(())
    }
}
