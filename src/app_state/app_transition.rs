pub enum AppTransition {
    None,
    StartConfiguringIdx(usize),
    AbortConfiguration,
    CommitConfiguration,
}
