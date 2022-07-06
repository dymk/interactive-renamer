
use regex::Regex;

use crate::{
    renamer::Renamer,
    utils::{file_name, split_ext},
};

pub const NUM_CONFIGS: usize = 5;
pub const NUM_SERIALIZED: usize = NUM_CONFIGS + 1;

const CONFIG_FILE_EXT: usize = 0;
const CONFIG_DIR_MATCHER: usize = 1;
const CONFIG_DIR_REPLACER: usize = 2;
const CONFIG_FILE_MATCHER: usize = 3;
const CONFIG_FILE_REPLACER: usize = 4;

#[derive(Clone)]
pub enum FileMapping {
    MappedTo { from_name: String, to_name: String },
    Filtered { name: String },
}

#[derive(Clone)]
pub struct MappedDir {
    in_dir_path: String,
    configs: [String; NUM_CONFIGS],

    // updated once upon construction
    in_file_list: Vec<String>,

    // updated when configs change
    file_mappings: Vec<FileMapping>,
    file_filter_regex: Option<Regex>,
    file_renamer: Option<Renamer>,
    dir_renamer: Option<Renamer>,
}

impl MappedDir {
    pub fn configs_eq(&self, other: &Self) -> bool {
        assert!(self.in_dir_path == other.in_dir_path);
        self.configs == other.configs
    }
}

impl MappedDir {
    pub fn deserialize(inputs: [String; NUM_SERIALIZED]) -> MappedDir {
        let [a, b, c, d, e, f] = inputs;
        let mut ret = MappedDir {
            in_dir_path: a,
            configs: [b, c, d, e, f],
            in_file_list: vec![],
            file_mappings: vec![],
            file_filter_regex: None,
            file_renamer: None,
            dir_renamer: None,
        };

        ret.load_input_file_list();
        ret.configs_changed();
        ret
    }

    fn load_input_file_list(&mut self) {
        self.in_file_list = std::fs::read_dir(&self.in_dir_path)
            .unwrap()
            .filter_map(|path| {
                let path = path.unwrap();
                let meta = path.metadata().unwrap();
                if meta.is_dir() {
                    return None;
                }
                if meta.is_symlink() {
                    return None;
                }
                Some(path.file_name().to_string_lossy().to_string())
            })
            .collect();
        self.in_file_list.sort();
    }

    fn configs_changed(&mut self) {
        self.file_filter_regex = build_file_filter_regex(self.configs[CONFIG_FILE_EXT].as_str());
        self.file_renamer = Renamer::new(
            self.configs[CONFIG_FILE_MATCHER].as_str(),
            self.configs[CONFIG_FILE_REPLACER].as_str(),
        );
        self.dir_renamer = Renamer::new(
            self.configs[CONFIG_DIR_MATCHER].as_str(),
            self.configs[CONFIG_DIR_REPLACER].as_str(),
        );

        self.file_mappings = self
            .in_file_list
            .iter()
            .filter_map(|path| {
                let path = path.clone();
                let (basename, ext) = split_ext(path.as_str());

                if let Some(ext_matcher) = &self.file_filter_regex {
                    if let Some(ext) = ext {
                        if !ext_matcher.is_match(ext) {
                            return Some(FileMapping::Filtered { name: path.clone() });
                        }
                    }
                }

                if let Some(file_renamer) = &self.file_renamer {
                    let renamed = file_renamer.process(basename);

                    let to_path = if let Some(ext) = ext {
                        format!("{}.{}", renamed, ext)
                    } else {
                        renamed
                    };

                    return Some(FileMapping::MappedTo {
                        from_name: path.clone(),
                        to_name: to_path,
                    });
                }

                return Some(FileMapping::MappedTo {
                    from_name: path.clone(),
                    to_name: path.clone(),
                });
            })
            .collect();
    }

    pub fn has_valid_file_filter(&self) -> bool {
        self.file_filter_regex.is_some()
    }
    pub fn has_valid_dir_renamer(&self) -> bool {
        self.dir_renamer.is_some()
    }
    pub fn has_valid_file_renamer(&self) -> bool {
        self.file_renamer.is_some()
    }

    pub fn serialize(&self) -> [&str; NUM_SERIALIZED] {
        [
            self.in_dir_path.as_str(),
            self.configs[0].as_str(),
            self.configs[1].as_str(),
            self.configs[2].as_str(),
            self.configs[3].as_str(),
            self.configs[4].as_str(),
        ]
    }

    pub fn config_mut<F>(&mut self, idx: usize, changer: &F)
    where
        F: Fn(&mut String),
    {
        changer(self.configs.get_mut(idx).unwrap());
        self.configs_changed();
    }

    pub fn config(&self, idx: usize) -> &String {
        self.configs.get(idx).unwrap()
    }
}

fn build_file_filter_regex(s: &str) -> Option<Regex> {
    let j = s
        .to_string()
        .split(",")
        .map(regex::escape)
        .collect::<Vec<_>>()
        .join("|");
    match Regex::new(&j) {
        Ok(r) => Some(r),
        Err(_) => None,
    }
}

impl MappedDir {
    pub fn out_dir_name(&self) -> Option<String> {
        self.dir_renamer
            .as_ref()
            .map(|renamer| renamer.process(self.in_dir_name()))
    }

    pub fn in_dir_name(&self) -> &str {
        file_name(self.in_dir_path.as_str())
    }

    pub fn in_dir_path(&self) -> &str {
        &self.in_dir_path
    }

    pub fn file_mappings(&self) -> &Vec<FileMapping> {
        &self.file_mappings
    }
}

pub enum MappingState {
    HasMapping { mapped_dir: MappedDir },
    Unmapped { in_path: String },
}
impl MappingState {
    pub fn in_dir_name(&self) -> &str {
        match self {
            MappingState::HasMapping { mapped_dir } => mapped_dir.in_dir_name(),
            MappingState::Unmapped { in_path } => file_name(in_path),
        }
    }

    pub fn to_mapped_dir(&self) -> MappedDir {
        match self {
            MappingState::HasMapping { mapped_dir } => mapped_dir.clone(),
            #[rustfmt::skip]
            MappingState::Unmapped { in_path } => MappedDir::deserialize(
                [
                    in_path.as_str(),
                    "avi,mkv,mp4", 
                    "(.+)", "$1", 
                    "(.+)", "$1"
                ].map(ToString::to_string),
            ),
        }
    }
}
