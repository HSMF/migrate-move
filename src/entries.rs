use std::{collections::HashMap, ffi::OsString, fs, io, ops::Index, path::Path};

use thiserror::Error;

use crate::{
    migration::{self, Migration},
    pattern::Pattern,
};

#[warn(missing_debug_implementations)]
#[derive(Debug)]
pub struct Entry {
    pub migration: Migration,
    pub name_up: String,
    pub name_down: String,
}

// i < j => self.entries[i].id < self.entries[j].id
#[derive(Debug)]
pub struct Entries<'a> {
    entries: Vec<Entry>,
    dir: &'a Path,
    pattern_up: Pattern<'a>,
    pattern_down: Pattern<'a>,
}

impl<'a> Entries<'a> {
    /// Returns the length of this [`Entries`].
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether this [`Entries`] is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Index<usize> for Entries<'_> {
    type Output = Entry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IntoIterator for Entries<'_> {
    type Item = Entry;

    type IntoIter = std::vec::IntoIter<Entry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("encountered io error {0}")]
    IoError(#[from] io::Error),
    #[error("file name {0:?} is not UTF-8. Currently only UTF-8 filenames are supported.")]
    FileNotUtf8(OsString),
    #[error("{0}")]
    InvalidFormat(migration::ParseError),
    #[error("the file {0} is missing its corresponding migration")]
    FileMissing(String),
}

fn select_pattern(
    pattern_up: &Pattern,
    pattern_down: &Pattern,
    name: &str,
) -> Result<(bool, Migration), migration::ParseError> {
    match Migration::from_pattern(pattern_up, name) {
        Err(migration::ParseError::IdOverflowing) => {
            return Err(migration::ParseError::IdOverflowing)
        }
        Ok(name) => return Ok((true, name)),
        Err(migration::ParseError::NotMatching) => {}
    }

    Migration::from_pattern(pattern_down, name).map(|ok| (false, ok))
}

impl<'a> Entries<'a> {
    pub fn from_dir<'b: 'a, 'c: 'a>(
        dir: &'b Path,
        pattern_up: Pattern<'c>,
        pattern_down: Pattern<'c>,
    ) -> Result<Entries<'a>, ReadError> {
        let readdir = dir.read_dir().map_err(ReadError::IoError)?;
        type HashMapT = HashMap<usize, (Option<(Migration, String)>, Option<(Migration, String)>)>;
        let mut entries: HashMapT = HashMap::new();

        for file in readdir {
            let file = file.map_err(ReadError::IoError)?;
            let metadata = file.metadata().map_err(ReadError::IoError)?;

            if metadata.is_file() {
                let name = file.file_name();
                let name = name
                    .to_str()
                    .ok_or_else(|| ReadError::FileNotUtf8(name.to_os_string()))?;
                let name = name.to_string();

                let migration = select_pattern(&pattern_up, &pattern_down, &name);
                match migration {
                    Ok((true, migration)) => {
                        if let Some(entry) = entries.get_mut(&migration.id) {
                            (*entry).0 = Some((migration, name));
                        } else {
                            entries.insert(migration.id, (Some((migration, name)), None));
                        }
                    }
                    Ok((false, migration)) => {
                        if let Some(entry) = entries.get_mut(&migration.id) {
                            (*entry).1 = Some((migration, name));
                        } else {
                            entries.insert(migration.id, (None, Some((migration, name))));
                        }
                    }
                    Err(x) => return Err(ReadError::InvalidFormat(x)),
                }
            }
        }

        let mut entries_vec = vec![];

        for (_, (up, down)) in entries {
            if up.is_none() || down.is_none() {
                let either = up.or(down).unwrap().0.name;
                return Err(ReadError::FileMissing(either));
            }
            let ((migration, name_up), (_, name_down)) = up.zip(down).unwrap();
            entries_vec.push(Entry {
                migration,
                name_up: name_up.to_string(),
                name_down: name_down.to_string(),
            })
        }

        let mut entries = entries_vec;
        entries.sort_by(|a, b| a.migration.id.cmp(&b.migration.id));

        Ok(Entries {
            entries,
            dir,
            pattern_up,
            pattern_down,
        })
    }

    pub fn write_back(self) -> Result<(), io::Error> {
        for entry in self.entries {
            let old_up = self.dir.join(entry.name_up);
            let new_up = self.dir.join(entry.migration.to_string(&self.pattern_up));
            fs::rename(old_up, new_up)?;

            let old_down = self.dir.join(entry.name_down);
            let new_down = self.dir.join(entry.migration.to_string(&self.pattern_down));
            fs::rename(old_down, new_down)?;
        }

        Ok(())
    }

    pub fn move_up(&mut self, i: usize) -> Option<()> {
        if i == 0 {
            return None;
        }

        let lower_id = self.entries[i - 1].migration.id;
        let higher_id = self.entries[i].migration.id;

        self.entries.swap(i, i - 1);

        self.entries[i - 1].migration.id = lower_id;
        self.entries[i].migration.id = higher_id;

        Some(())
    }

    pub fn move_down(&mut self, i: usize) -> Option<()> {
        if i + 1 >= self.entries.len() {
            return None;
        }

        let lower_id = self.entries[i + 1].migration.id;
        let higher_id = self.entries[i].migration.id;

        self.entries.swap(i, i + 1);

        self.entries[i + 1].migration.id = lower_id;
        self.entries[i].migration.id = higher_id;

        Some(())
    }
}
