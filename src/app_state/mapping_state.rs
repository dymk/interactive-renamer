use std::path::PathBuf;

use regex::Regex;

use crate::renamer::Renamer;

pub const NUM_CONFIG_INPUTS: usize = 5;

pub struct MappedDir {
    in_path: PathBuf,
    file_filter: Regex,
    dir_renamer: Renamer,
    file_renamer: Renamer,
}
impl MappedDir {
    pub fn from_inputs(in_path: PathBuf, inputs: [String; NUM_CONFIG_INPUTS]) -> MappedDir {
        MappedDir {
            in_path,
            file_filter: Regex::new(&inputs[0]).unwrap(),
            dir_renamer: Renamer::new(inputs[1].as_str(), inputs[2].as_str()),
            file_renamer: Renamer::new(inputs[3].as_str(), inputs[4].as_str()),
        }
    }

    pub fn to_inputs(&self) -> [String; NUM_CONFIG_INPUTS] {
        [
            self.file_filter.as_str().to_string(),
            self.dir_renamer.get_finder_str(),
            self.dir_renamer.get_replacer_str(),
            self.file_renamer.get_finder_str(),
            self.file_renamer.get_replacer_str(),
        ]
    }
}

impl MappedDir {
    pub fn in_path(&self) -> &str {
        &self.in_path.as_os_str().to_str().unwrap()
    }
    pub fn out_path(&self) -> String {
        self.dir_renamer.process(self.in_name())
    }

    fn in_name(&self) -> &str {
        let in_path = self.in_path();
        if let Some(sep) = in_path.rfind("/") {
            &in_path[sep..]
        } else {
            in_path
        }
    }

    fn in_dirname(&self) -> &str {
        let in_path = self.in_path();
        if let Some(sep) = in_path.rfind("/") {
            &in_path[..sep]
        } else {
            ""
        }
    }
}

pub enum MappingState {
    HasMapping(MappedDir),
    Unmapped(String),
}
impl MappingState {
    pub fn in_path(&self) -> &str {
        match self {
            MappingState::HasMapping(mapping) => mapping.in_path(),
            MappingState::Unmapped(in_path) => in_path.as_str(),
        }
    }

    pub fn to_inputs(&self) -> [String; NUM_CONFIG_INPUTS] {
        match self {
            MappingState::HasMapping(m) => m.to_inputs(),
            #[rustfmt::skip]
            MappingState::Unmapped(_) => [
                "avi,mkv,mp4",
                "(.+)", "$1",
                "(.+)", "$1"
            ].map(ToString::to_string),
        }
    }
}
