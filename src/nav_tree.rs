// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    Element,
    iced::{Length, clipboard::dnd::DndAction},
    theme,
    widget::{self, DndDestination, icon},
};
use rustc_hash::FxHashSet;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{clipboard::ClipboardPaste, fl, home_dir, tab::folder_icon_symbolic};

#[derive(Clone, Debug)]
pub struct TreeNode {
    pub path: PathBuf,
    pub name: String,
    pub expanded: bool,
    pub children: Vec<TreeNode>,
    pub depth: usize,
}

impl TreeNode {
    pub fn new(path: PathBuf, depth: usize) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(String::from)
            .unwrap_or_else(|| {
                if path == Path::new("/") {
                    fl!("filesystem")
                } else {
                    path.display().to_string()
                }
            });

        Self {
            path,
            name,
            expanded: false,
            children: Vec::new(),
            depth,
        }
    }

    pub fn new_with_name(path: PathBuf, name: String, depth: usize) -> Self {
        Self {
            path,
            name,
            expanded: false,
            children: Vec::new(),
            depth,
        }
    }

    fn has_subdirectories(&self) -> bool {
        if let Ok(entries) = fs::read_dir(&self.path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        return true;
                    }
                }
            }
        }
        false
    }
}

#[derive(Clone, Debug, Default)]
pub struct NavTreeState {
    pub roots: Vec<TreeNode>,
    pub expanded_paths: FxHashSet<PathBuf>,
    pub current_path: Option<PathBuf>,
}

impl NavTreeState {
    pub fn new() -> Self {
        let home = home_dir();
        let root = PathBuf::from("/");

        let mut roots = vec![
            TreeNode::new_with_name(home, fl!("home"), 0),
            TreeNode::new_with_name(root, fl!("filesystem"), 0),
        ];

        // Add mounted volumes
        if let Ok(entries) = fs::read_dir("/media") {
            for entry in entries.flatten() {
                if entry.file_type().is_ok_and(|ft| ft.is_dir()) {
                    roots.push(TreeNode::new(entry.path(), 0));
                }
            }
        }

        // Add /run/media/$USER mounts
        if let Some(user) = std::env::var_os("USER") {
            let user_media = PathBuf::from("/run/media").join(user);
            if let Ok(entries) = fs::read_dir(&user_media) {
                for entry in entries.flatten() {
                    if entry.file_type().is_ok_and(|ft| ft.is_dir()) {
                        roots.push(TreeNode::new(entry.path(), 0));
                    }
                }
            }
        }

        Self {
            roots,
            expanded_paths: FxHashSet::default(),
            current_path: None,
        }
    }

    pub fn toggle_expand(&mut self, path: &PathBuf) {
        if self.expanded_paths.contains(path) {
            self.expanded_paths.remove(path);
            self.collapse_node(path);
        } else {
            self.expanded_paths.insert(path.clone());
            self.load_children(path);
        }
    }

    fn collapse_node(&mut self, path: &PathBuf) {
        fn collapse_in_nodes(nodes: &mut [TreeNode], path: &PathBuf) {
            for node in nodes.iter_mut() {
                if node.path == *path {
                    node.expanded = false;
                    node.children.clear();
                    return;
                }
                collapse_in_nodes(&mut node.children, path);
            }
        }
        collapse_in_nodes(&mut self.roots, path);
    }

    fn load_children(&mut self, path: &PathBuf) {
        fn load_in_nodes(nodes: &mut [TreeNode], path: &PathBuf) -> bool {
            for node in nodes.iter_mut() {
                if node.path == *path {
                    node.expanded = true;
                    node.children = scan_children(&node.path, node.depth + 1);
                    return true;
                }
                if load_in_nodes(&mut node.children, path) {
                    return true;
                }
            }
            false
        }
        load_in_nodes(&mut self.roots, path);
    }

    pub fn expand_to_path(&mut self, target: &PathBuf) {
        self.current_path = Some(target.clone());

        // Find which root this path belongs to
        let matching_root = self
            .roots
            .iter()
            .filter(|r| target.starts_with(&r.path))
            .max_by_key(|r| r.path.components().count());

        let Some(root) = matching_root else {
            return;
        };

        let root_path = root.path.clone();

        // Build the path from root to target
        let relative = match target.strip_prefix(&root_path) {
            Ok(rel) => rel,
            Err(_) => return,
        };

        // Expand the root first
        if !self.expanded_paths.contains(&root_path) {
            self.expanded_paths.insert(root_path.clone());
            self.load_children(&root_path);
        }

        // Expand each component of the path, including the target itself
        let mut current = root_path;
        for component in relative.components() {
            let next_path = current.join(component);

            // Ensure hidden directories are added to tree if navigating through them
            self.ensure_child_exists(&current, &next_path);

            current = next_path;
            if !self.expanded_paths.contains(&current) {
                self.expanded_paths.insert(current.clone());
                self.load_children(&current);
            }
        }
    }

    fn ensure_child_exists(&mut self, parent: &PathBuf, child: &PathBuf) {
        fn ensure_in_nodes(nodes: &mut [TreeNode], parent: &PathBuf, child: &PathBuf) -> bool {
            for node in nodes.iter_mut() {
                if node.path == *parent {
                    // Check if child already exists
                    if !node.children.iter().any(|c| c.path == *child) {
                        // Add hidden directory to children
                        if child.is_dir() {
                            let depth = node.depth + 1;
                            let new_node = TreeNode::new(child.clone(), depth);
                            // Insert in sorted position
                            let insert_pos = node
                                .children
                                .iter()
                                .position(|c| c.name.to_lowercase() > new_node.name.to_lowercase())
                                .unwrap_or(node.children.len());
                            node.children.insert(insert_pos, new_node);
                        }
                    }
                    return true;
                }
                if ensure_in_nodes(&mut node.children, parent, child) {
                    return true;
                }
            }
            false
        }
        ensure_in_nodes(&mut self.roots, parent, child);
    }

    /// Calculate the scroll offset to bring the current path into view.
    /// Accounts for variable item heights due to text wrapping.
    pub fn scroll_offset_for_current(&self) -> f32 {
        let Some(target) = &self.current_path else {
            return 0.0;
        };

        let base_item_height = 32.0;
        let char_width = 8.0; // Approximate character width
        let pane_width = 260.0; // Approximate usable width for text (280 - indent - icons)
        let mut total_height = 40.0; // Start with toggle row height

        fn calc_height_before(
            nodes: &[TreeNode],
            target: &PathBuf,
            expanded: &FxHashSet<PathBuf>,
            total: &mut f32,
            base_height: f32,
            char_width: f32,
            pane_width: f32,
        ) -> bool {
            for node in nodes {
                if node.path == *target {
                    return true; // Found it
                }
                // Calculate height for this item, accounting for text wrap
                let indent = node.depth as f32 * 16.0;
                let available_width = (pane_width - indent - 50.0).max(50.0); // 50px for icons
                let text_width = node.name.len() as f32 * char_width;
                let lines = (text_width / available_width).ceil().max(1.0);
                *total += base_height * lines;

                // If this node is expanded, count its children too
                if expanded.contains(&node.path) {
                    if calc_height_before(
                        &node.children,
                        target,
                        expanded,
                        total,
                        base_height,
                        char_width,
                        pane_width,
                    ) {
                        return true;
                    }
                }
            }
            false
        }

        calc_height_before(
            &self.roots,
            target,
            &self.expanded_paths,
            &mut total_height,
            base_item_height,
            char_width,
            pane_width,
        );

        total_height.max(0.0)
    }

    pub fn view<'a, Message: Clone + 'static>(
        &'a self,
        dnd_hover: Option<&PathBuf>,
        on_toggle: impl Fn(PathBuf) -> Message + Clone + 'static,
        on_navigate: impl Fn(PathBuf) -> Message + Clone + 'static,
        on_dnd_enter: impl Fn(PathBuf) -> Message + Clone + 'static,
        on_dnd_leave: Message,
        on_dnd_drop: impl Fn(PathBuf, Option<ClipboardPaste>, DndAction) -> Message + Clone + 'static,
    ) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let mut items: Vec<Element<'a, Message>> = Vec::new();

        for root in &self.roots {
            self.render_node(
                root,
                dnd_hover,
                &on_toggle,
                &on_navigate,
                &on_dnd_enter,
                &on_dnd_leave,
                &on_dnd_drop,
                &mut items,
            );
        }

        widget::column::with_children(items)
            .spacing(spacing.space_xxs)
            .width(Length::Fill)
            .into()
    }

    #[allow(clippy::too_many_arguments)]
    fn render_node<'a, Message: Clone + 'static>(
        &'a self,
        node: &'a TreeNode,
        dnd_hover: Option<&PathBuf>,
        on_toggle: &(impl Fn(PathBuf) -> Message + Clone + 'static),
        on_navigate: &(impl Fn(PathBuf) -> Message + Clone + 'static),
        on_dnd_enter: &(impl Fn(PathBuf) -> Message + Clone + 'static),
        on_dnd_leave: &Message,
        on_dnd_drop: &(impl Fn(PathBuf, Option<ClipboardPaste>, DndAction) -> Message + Clone + 'static),
        items: &mut Vec<Element<'a, Message>>,
    ) {
        let spacing = theme::active().cosmic().spacing;
        let is_expanded = self.expanded_paths.contains(&node.path);
        let is_current = self.current_path.as_ref().is_some_and(|p| *p == node.path);
        let is_dnd_hovered = dnd_hover.is_some_and(|p| *p == node.path);
        let has_children = node.has_subdirectories();
        let indent = (node.depth as u16) * 16;

        // Toggle icon (+/-)
        let toggle_icon = if has_children {
            let icon_name = if is_expanded {
                "pan-down-symbolic"
            } else {
                "pan-end-symbolic"
            };
            let path_clone = node.path.clone();
            let on_toggle_clone = on_toggle.clone();
            widget::button::icon(icon::from_name(icon_name).size(16))
                .padding(0)
                .on_press(on_toggle_clone(path_clone))
                .class(theme::Button::Icon)
                .into()
        } else {
            widget::Space::with_width(Length::Fixed(16.0)).into()
        };

        // Folder icon
        let folder_icon: Element<'a, Message> =
            icon::icon(folder_icon_symbolic(&node.path, 16)).into();

        // Name with navigation
        let path_clone = node.path.clone();
        let on_navigate_clone = on_navigate.clone();
        let name_button =
            widget::button::custom(widget::text::body(&node.name).width(Length::Fill))
                .padding([spacing.space_xxxs, spacing.space_xs])
                .on_press(on_navigate_clone(path_clone))
                .class(if is_current || is_dnd_hovered {
                    theme::Button::Suggested
                } else {
                    theme::Button::Text
                });

        let row = widget::row::with_children(vec![
            widget::Space::with_width(Length::Fixed(indent as f32)).into(),
            toggle_icon,
            folder_icon,
            name_button.into(),
        ])
        .spacing(spacing.space_xxs)
        .align_y(cosmic::iced::Alignment::Center);

        // Wrap with DndDestination for drag-and-drop support
        let path_for_drop = node.path.clone();
        let path_for_enter = node.path.clone();
        let on_dnd_enter_clone = on_dnd_enter.clone();
        let on_dnd_leave_clone = on_dnd_leave.clone();
        let on_dnd_drop_clone = on_dnd_drop.clone();

        let dnd_row = DndDestination::for_data::<ClipboardPaste>(row, move |data, action| {
            on_dnd_drop_clone(path_for_drop.clone(), data, action)
        })
        .on_enter(move |_, _, _| on_dnd_enter_clone(path_for_enter.clone()))
        .on_leave(move || on_dnd_leave_clone.clone());

        items.push(dnd_row.into());

        // Render children if expanded
        if is_expanded {
            for child in &node.children {
                self.render_node(
                    child,
                    dnd_hover,
                    on_toggle,
                    on_navigate,
                    on_dnd_enter,
                    on_dnd_leave,
                    on_dnd_drop,
                    items,
                );
            }
        }
    }
}

fn scan_children(path: &Path, depth: usize) -> Vec<TreeNode> {
    let mut children = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        let mut dirs: Vec<_> = entries
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|ft| ft.is_dir()))
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .is_some_and(|name| !name.starts_with('.'))
            })
            .collect();

        dirs.sort_by(|a, b| {
            a.file_name()
                .to_ascii_lowercase()
                .cmp(&b.file_name().to_ascii_lowercase())
        });

        for entry in dirs {
            children.push(TreeNode::new(entry.path(), depth));
        }
    }

    children
}
