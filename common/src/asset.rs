use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::{cache::manifest_dir, FileOptions};

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub enum AssetType {
    File(FileAsset),
    Tailwind(TailwindAsset),
    Metadata(MetadataAsset),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub enum FileSource {
    Local(PathBuf),
    Remote(Url),
}

impl FileSource {
    pub fn last_segment(&self) -> &str {
        match self {
            Self::Local(path) => path.file_name().unwrap().to_str().unwrap(),
            Self::Remote(url) => url.path_segments().unwrap().last().unwrap(),
        }
    }

    pub fn extension(&self) -> Option<&str> {
        match self {
            Self::Local(path) => path.extension().map(|e| e.to_str().unwrap()),
            Self::Remote(url) => url
                .path_segments()
                .unwrap()
                .last()
                .unwrap()
                .split('.')
                .last(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct FileLocation {
    unique_name: String,
    source: FileSource,
}

impl FileLocation {
    pub fn unique_name(&self) -> &str {
        &self.unique_name
    }

    pub fn source(&self) -> &FileSource {
        &self.source
    }

    pub fn read_to_string(&self) -> anyhow::Result<String> {
        match &self.source {
            FileSource::Local(path) => Ok(std::fs::read_to_string(path)?),
            FileSource::Remote(url) => {
                let response = reqwest::blocking::get(url.as_str())?;
                Ok(response.text()?)
            }
        }
    }

    pub fn read_to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        match &self.source {
            FileSource::Local(path) => Ok(std::fs::read(path)?),
            FileSource::Remote(url) => {
                let response = reqwest::blocking::get(url.as_str())?;
                Ok(response.bytes().map(|b| b.to_vec())?)
            }
        }
    }
}

impl FromStr for FileSource {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Url::parse(s) {
            Ok(url) => Ok(Self::Remote(url)),
            Err(_) => {
                let manifest_dir = manifest_dir();
                let path = manifest_dir.join(PathBuf::from(s));
                Ok(Self::Local(path.canonicalize()?))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct FileAsset {
    location: FileLocation,
    options: FileOptions,
}

impl FileAsset {
    pub fn new(source: FileSource) -> std::io::Result<Self> {
        let options = FileOptions::default_for_extension(source.extension());
        Self::new_with_options(source, options)
    }

    pub fn new_with_options(source: FileSource, options: FileOptions) -> std::io::Result<Self> {
        let manifest_dir = manifest_dir();
        let path = manifest_dir.join(source.last_segment());
        let uuid = uuid::Uuid::new_v4();
        let file_name = path.file_stem().unwrap().to_string_lossy();
        let extension = options
            .extension()
            .map(|e| format!(".{e}"))
            .unwrap_or_default();
        let uuid_hex = uuid.simple().to_string();
        let unique_name = format!("{file_name}{uuid_hex}{extension}");

        Ok(Self {
            location: FileLocation {
                unique_name,
                source,
            },
            options,
        })
    }

    pub fn process_file(&self, output_folder: &Path) -> anyhow::Result<()> {
        self.options.process_file(&self.location, output_folder)?;

        Ok(())
    }

    pub fn set_unique_name(&mut self, unique_name: &str) {
        self.location.unique_name = unique_name.to_string();
    }

    pub fn unique_name(&self) -> &str {
        &self.location.unique_name
    }

    pub fn source(&self) -> &FileSource {
        &self.location.source
    }

    pub fn options(&self) -> &FileOptions {
        &self.options
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct MetadataAsset {
    key: String,
    value: String,
}

impl MetadataAsset {
    pub fn new(key: &str, value: &str) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct TailwindAsset {
    classes: String,
}

impl TailwindAsset {
    pub fn new(classes: &str) -> Self {
        Self {
            classes: classes.to_string(),
        }
    }

    pub fn classes(&self) -> &str {
        &self.classes
    }
}
