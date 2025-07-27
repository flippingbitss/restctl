use std::sync::Arc;

use egui::text::CCursorRange;

/// The output from a [`TextEdit`](crate::TextEdit).
pub struct TextEditOutput {
    /// The interaction response.
    pub response: egui::Response,

    /// How the text was displayed.
    pub galley: Arc<egui::Galley>,

    /// Where the text in [`Self::galley`] ended up on the screen.
    pub galley_pos: egui::Pos2,

    /// The text was clipped to this rectangle when painted.
    pub text_clip_rect: egui::Rect,

    /// The state we stored after the run.
    pub state: super::state::TextEditState,

    /// Where the text cursor is.
    pub cursor_range: Option<CCursorRange>,
}

// TODO(emilk): add `output.paint` and `output.store` and split out that code from `TextEdit::show`.
