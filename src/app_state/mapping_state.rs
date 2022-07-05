use crate::renamer::Renamer;

pub const NUM_CONFIGS: usize = 5;
pub const NUM_SERIALIZED: usize = NUM_CONFIGS + 1;

const CONFIG_FILE_EXT: usize = 0;
const CONFIG_DIR_MATCHER: usize = 1;
const CONFIG_DIR_REPLACER: usize = 2;
const CONFIG_FILE_MATCHER: usize = 3;
const CONFIG_FILE_REPLACER: usize = 4;

#[derive(Clone, Eq, PartialEq)]
pub struct MappedDir {
    in_path: String,
    configs: [String; NUM_CONFIGS],
}

impl MappedDir {
    pub fn deserialize(inputs: [String; NUM_SERIALIZED]) -> MappedDir {
        let [a, b, c, d, e, f] = inputs;
        MappedDir {
            in_path: a,
            configs: [b, c, d, e, f],
        }
    }

    pub fn serialize(&self) -> [&str; NUM_SERIALIZED] {
        [
            self.in_path.as_str(),
            self.configs[0].as_str(),
            self.configs[1].as_str(),
            self.configs[2].as_str(),
            self.configs[3].as_str(),
            self.configs[4].as_str(),
        ]
    }

    pub fn config_mut(&mut self, idx: usize) -> &mut String {
        self.configs.get_mut(idx).unwrap()
    }
    pub fn config(&self, idx: usize) -> &String {
        self.configs.get(idx).unwrap()
    }
}

impl MappedDir {
    pub fn in_path(&self) -> &str {
        &self.in_path.as_str()
    }

    pub fn out_path(&self) -> String {
        Renamer::new(
            &self.config(CONFIG_DIR_MATCHER),
            &self.config(CONFIG_DIR_REPLACER),
        )
        .process(self.in_name())
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

    pub fn to_mapped_dir(&self) -> MappedDir {
        match self {
            MappingState::HasMapping(mapping) => mapping.clone(),
            #[rustfmt::skip]
            MappingState::Unmapped(in_path) => MappedDir::deserialize(
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
