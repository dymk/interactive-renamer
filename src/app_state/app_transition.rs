use super::mapping_state::MappedDir;

pub enum AppTransition {
    None,
    Quit,
    StartConfiguringIdx(usize),
    AbortConfiguration,
    CommitConfiguration(usize, MappedDir),
}
