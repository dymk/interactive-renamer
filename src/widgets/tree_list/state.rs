use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct TreeListState {
    pub(super) changed: bool,
    pub(super) focused: bool,
    pub(super) collapsed_paths: HashSet<String>,
    pub(super) scroll_offset: usize,
    pub(super) focused_item: usize,
}

// impl Default for TreeListState {
//     fn default() -> Self {
//         Self {
//             changed: false,
//             focused: false,
//             collapsed_paths: Default::default(),
//             scroll_position: 0,
//         }
//     }
// }
