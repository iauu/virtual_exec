// source: https://github.com/Brainwires/ratatui-interact/blob/main/src/components/tree_view.rs

use std::collections::HashSet;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};
use virtual_exec_type::base::TypeCast;
use virtual_exec_type::vm_type::Any;

/// A node in the tree
#[derive(Debug, Clone)]
pub struct TreeNode<'b> {
    /// Unique identifier for this node
    pub id: String,
    pub name: String,
    /// The data associated with this node
    pub data: Any<'b>,
}

impl<'b> TreeNode<'b> {
    /// Create a new tree node
    pub fn new(id: impl Into<String>, name: String, data: Any<'b>) -> Self {
        Self {
            id: id.into(),
            name,
            data,
        }
    }

    /// Check if this node has children
    pub fn has_children(&self) -> bool {
        if let Some(obj) = self.data.as_object() {
            obj.read_arc_blocking().len() > 0
        } else if let Some(arr) = self.data.as_collections() {
            arr.read_arc_blocking().len() > 0
        } else {
            false
        }
    }

    pub fn get_children(&self) -> Vec<TreeNode<'b>> {
        if let Some(obj) = self.data.as_object() {
            let pre_id = format!("{}.", self.id);
            obj.read_arc_blocking()
                .iter()
                .map(|x| TreeNode::new(pre_id.clone() + x.0, x.0.clone(), x.1.clone()))
                .collect()
        } else if let Some(arr) = self.data.as_collections() {
            let pre_id = format!("{}.", self.id);
            arr.read_arc_blocking()
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    TreeNode::new(
                        pre_id.clone() + &i.to_string(),
                        format!("[{}]", &i),
                        x.clone(),
                    )
                })
                .collect()
        } else {
            vec![]
        }
    }
}

/// State for the tree view widget
#[derive(Debug, Clone, Default)]
pub struct TreeViewState {
    /// Set of collapsed node IDs
    pub collapsed: HashSet<String>,
    /// Currently selected index in the flattened visible list
    pub selected_index: usize,
    /// Scroll offset
    pub scroll: u16,
}

impl TreeViewState {
    /// Create a new tree view state
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle the collapsed state of a node
    pub fn toggle_collapsed(&mut self, id: &str) {
        if self.collapsed.contains(id) {
            self.collapsed.remove(id);
        } else {
            self.collapsed.insert(id.to_string());
        }
    }

    /// Check if a node is collapsed
    pub fn is_collapsed(&self, id: &str) -> bool {
        self.collapsed.contains(id)
    }

    /// Collapse a node
    pub fn collapse(&mut self, id: &str) {
        self.collapsed.insert(id.to_string());
    }

    /// Expand a node
    pub fn expand(&mut self, id: &str) {
        self.collapsed.remove(id);
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    /// Move selection down (needs total count)
    pub fn select_next(&mut self, total_visible: usize) {
        if self.selected_index + 1 < total_visible {
            self.selected_index += 1;
        }
    }

    /// Ensure selection is visible given viewport height
    pub fn ensure_visible(&mut self, viewport_height: usize) {
        if self.selected_index < self.scroll as usize {
            self.scroll = self.selected_index as u16;
        } else if self.selected_index >= self.scroll as usize + viewport_height {
            self.scroll = (self.selected_index - viewport_height + 1) as u16;
        }
    }
}

/// Style configuration for tree view
#[derive(Debug, Clone)]
pub struct TreeStyle {
    /// Style for selected items
    pub selected_style: Style,
    /// Style for normal items
    pub normal_style: Style,
    /// Style for tree connectors
    pub connector_style: Style,
    /// Style for expand/collapse icons
    pub icon_style: Style,
    /// Collapsed icon
    pub collapsed_icon: &'static str,
    /// Expanded icon
    pub expanded_icon: &'static str,
    /// Tree connector: branch (has siblings after)
    pub connector_branch: &'static str,
    /// Tree connector: last (no siblings after)
    pub connector_last: &'static str,
    /// Tree connector: vertical line
    pub connector_vertical: &'static str,
    /// Tree connector: empty space
    pub connector_space: &'static str,
    /// Selection cursor for selected item
    pub cursor_selected: &'static str,
    /// Selection cursor for non-selected items
    pub cursor_normal: &'static str,
}

impl Default for TreeStyle {
    fn default() -> Self {
        Self {
            selected_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            normal_style: Style::default().fg(Color::White),
            connector_style: Style::default().fg(Color::DarkGray),
            icon_style: Style::default().fg(Color::Cyan),
            collapsed_icon: "▶ ",
            expanded_icon: "▼ ",
            connector_branch: "├── ",
            connector_last: "└── ",
            connector_vertical: "│   ",
            connector_space: "    ",
            cursor_selected: "> ",
            cursor_normal: "  ",
        }
    }
}

impl From<&ratatui_interact::theme::Theme> for TreeStyle {
    fn from(theme: &ratatui_interact::theme::Theme) -> Self {
        let p = &theme.palette;
        Self {
            selected_style: Style::default().fg(p.primary).add_modifier(Modifier::BOLD),
            normal_style: Style::default().fg(p.text),
            connector_style: Style::default().fg(p.text_disabled),
            icon_style: Style::default().fg(p.secondary),
            collapsed_icon: "▶ ",
            expanded_icon: "▼ ",
            connector_branch: "├── ",
            connector_last: "└── ",
            connector_vertical: "│   ",
            connector_space: "    ",
            cursor_selected: "> ",
            cursor_normal: "  ",
        }
    }
}

impl TreeStyle {
    /// Create a minimal style without tree connectors
    pub fn minimal() -> Self {
        Self {
            connector_branch: "  ",
            connector_last: "  ",
            connector_vertical: "  ",
            connector_space: "  ",
            ..Default::default()
        }
    }
}

/// Flattened node info for rendering
#[derive(Debug, Clone)]
pub struct FlatNode<'b> {
    /// Reference to the original node
    pub node: TreeNode<'b>,
    /// Depth in the tree (0 = root)
    pub depth: usize,
    /// Whether this is the last sibling at its level
    pub is_last: bool,
    /// Path of is_last values from root to parent
    pub parent_is_last: Vec<bool>,
}

/// Tree view widget
pub struct TreeView<'a, 'b, F>
where
    F: Fn(&TreeNode<'b>, bool) -> String,
{
    nodes: Vec<TreeNode<'b>>,
    state: &'a TreeViewState,
    style: TreeStyle,
    render_fn: F,
}

impl<'a, 'b> TreeView<'a, 'b, fn(&TreeNode<'b>, bool) -> String> {
    /// Create a new tree view with default rendering
    pub fn new(nodes: Vec<TreeNode<'b>>, state: &'a TreeViewState) -> Self {
        Self {
            nodes,
            state,
            style: TreeStyle::default(),
            render_fn: |node, _| format!("{:?}", node.id),
        }
    }
}

impl<'a, 'b, F> TreeView<'a, 'b, F>
where
    F: Fn(&TreeNode<'b>, bool) -> String,
{
    /// Set the render function for items
    pub fn render_item<G>(self, render_fn: G) -> TreeView<'a, 'b, G>
    where
        G: Fn(&TreeNode<'b>, bool) -> String,
    {
        TreeView {
            nodes: self.nodes,
            state: self.state,
            style: self.style,
            render_fn,
        }
    }

    /// Set the style
    pub fn style(mut self, style: TreeStyle) -> Self {
        self.style = style;
        self
    }

    /// Apply a theme to derive the style
    pub fn theme(self, theme: &ratatui_interact::theme::Theme) -> Self {
        self.style(TreeStyle::from(theme))
    }

    /// Flatten the tree into a list of visible nodes
    fn flatten_visible(&self) -> Vec<FlatNode<'b>> {
        let mut result = Vec::new();
        self.flatten_nodes(self.nodes.clone(), 0, &mut result, &[]);
        result
    }

    fn flatten_nodes(
        &self,
        nodes: Vec<TreeNode<'b>>,
        depth: usize,
        result: &mut Vec<FlatNode<'b>>,
        parent_is_last: &[bool],
    ) {
        let count = nodes.len();
        for (idx, node) in nodes.iter().enumerate() {
            let is_last = idx == count - 1;
            result.push(FlatNode {
                node: node.clone(),
                depth,
                is_last,
                parent_is_last: parent_is_last.to_vec(),
            });

            // Only recurse into children if not collapsed
            if node.has_children() && !self.state.is_collapsed(&node.id) {
                let mut new_parent_is_last = parent_is_last.to_vec();
                new_parent_is_last.push(is_last);
                self.flatten_nodes(node.get_children(), depth + 1, result, &new_parent_is_last);
            }
        }
    }

    /// Get the total number of visible nodes
    pub fn visible_count(&self) -> usize {
        self.flatten_visible().len()
    }

    /// Build the lines for rendering
    fn build_lines(&self, area: Rect) -> Vec<Line<'static>> {
        let visible = self.flatten_visible();
        let mut lines = Vec::new();

        let scroll = self.state.scroll as usize;
        let viewport_height = area.height as usize;

        for (idx, flat_node) in visible
            .iter()
            .enumerate()
            .skip(scroll)
            .take(viewport_height)
        {
            let is_selected = idx == self.state.selected_index;
            let mut spans = Vec::new();

            // Selection cursor
            let cursor = if is_selected {
                self.style.cursor_selected
            } else {
                self.style.cursor_normal
            };
            spans.push(Span::styled(
                cursor.to_string(),
                if is_selected {
                    self.style.selected_style
                } else {
                    self.style.normal_style
                },
            ));

            // Tree connectors
            for &parent_is_last in flat_node.parent_is_last.iter() {
                let connector = if parent_is_last {
                    self.style.connector_space
                } else {
                    self.style.connector_vertical
                };
                spans.push(Span::styled(
                    connector.to_string(),
                    self.style.connector_style,
                ));
            }

            // Branch connector for this node (if not root)
            if flat_node.depth > 0 {
                let connector = if flat_node.is_last {
                    self.style.connector_last
                } else {
                    self.style.connector_branch
                };
                spans.push(Span::styled(
                    connector.to_string(),
                    self.style.connector_style,
                ));
            }

            // Expand/collapse icon (if has children)
            if flat_node.node.has_children() {
                let icon = if self.state.is_collapsed(&flat_node.node.id) {
                    self.style.collapsed_icon
                } else {
                    self.style.expanded_icon
                };
                spans.push(Span::styled(icon.to_string(), self.style.icon_style));
            }

            // Node content
            let content = (self.render_fn)(&flat_node.node, is_selected);
            spans.push(Span::styled(
                content,
                if is_selected {
                    self.style.selected_style
                } else {
                    self.style.normal_style
                },
            ));

            lines.push(Line::from(spans));
        }

        lines
    }
}

impl<'a, 'b, F> Widget for TreeView<'a, 'b, F>
where
    F: Fn(&TreeNode<'b>, bool) -> String,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines = self.build_lines(area);
        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        paragraph.render(area, buf);
    }
}

/// Get the selected node ID from a tree view state and nodes
pub fn get_selected_id(nodes: Vec<TreeNode>, state: &TreeViewState) -> Option<String> {
    let tree = TreeView::new(nodes, state);
    let visible = tree.flatten_visible();
    visible.get(state.selected_index).map(|f| f.node.id.clone())
}
