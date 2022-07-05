use super::mapping_state::MappedDir;

pub enum AppTransition {
    None,
    StartConfiguringIdx(usize),
    AbortConfiguration,
    CommitConfiguration(usize, MappedDir),
}
