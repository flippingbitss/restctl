use std::{ops::Range, sync::Arc};

use egui::{Color32, FontId, Mesh, Pos2, Rect, Stroke, Vec2, pos2, text::CCursor, vec2};
use ropey::RopeSlice;

// use crate::code::text_layout_types::Glyph;
//
// #[derive(Default, Clone, Debug, PartialEq)]
// pub struct SingleLineLayoutJob<'a> {
//     pub text: &'a str,
//     pub runs: Vec<TextRun>,
//
//     pub char_width: f32,
//     pub line_height: f32,
//     pub len_bytes: usize,
// }
//

#[derive(Clone, Debug, PartialEq)]
pub struct TextFormat {
    // Text color
    pub color: Color32,

    pub background_color: Color32,

    pub italics: bool,

    pub underline: Stroke,

    pub strikethrough: Stroke,
    pub expand_bg: f32,
}

impl TextFormat {
    fn simple(color: Color32) -> Self {
        Self {
            color,
            ..Default::default()
        }
    }
}

impl Default for TextFormat {
    fn default() -> Self {
        Self {
            color: Color32::WHITE,
            background_color: Color32::TRANSPARENT,
            italics: false,
            strikethrough: Stroke::NONE,
            underline: Stroke::NONE,
            expand_bg: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LineSection {
    pub byte_range: Range<usize>,
    pub char_range: Range<usize>,
    pub format: TextFormat,
}

/// Describes a single line layout job
#[derive(Clone, Debug, PartialEq)]
pub struct LineLayoutJob<'a> {
    /// The rope slice for this line
    pub rope: RopeSlice<'a>,

    /// Different formatting sections within the line
    pub sections: Vec<LineSection>,
    // /// Starting column offset (for horizontal scrolling)
    // pub start_col: usize,
    //
    // /// Maximum visible columns (for horizontal scrolling)
    // pub max_cols: Option<usize>,
}

impl<'a> LineLayoutJob<'a> {
    /// Create a simple single-format line
    pub fn simple(rope: RopeSlice<'a>, format: TextFormat) -> Self {
        Self {
            sections: vec![LineSection {
                char_range: 0..(rope.len_chars()),
                byte_range: 0..(rope.len_bytes()),
                format,
            }],
            rope,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }

    // /// Get the visible character range based on scrolling
    // pub fn visible_range(&self) -> Range<usize> {
    //     let line_len = self.rope.len_chars();
    //     let start = self.start_col.min(line_len);
    //     let end = if let Some(max_cols) = self.max_cols {
    //         (start + max_cols).min(line_len)
    //     } else {
    //         line_len
    //     };
    //     start..end
    // }
}

/// A positioned glyph in the layout
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Glyph {
    /// The character this glyph represents
    pub chr: char,

    /// Position relative to the line start
    pub pos: Pos2,

    /// Character advance width (monospace, so same for all chars)
    pub advance_width: f32,

    /// Font height information
    pub font_height: f32,
    pub font_ascent: f32,

    /// UV coordinates in font texture
    pub uv_rect: super::text_layout_types::UvRect,

    /// Index into the runs array of the LineLayoutJob
    pub run_index: usize,
}

impl Glyph {
    #[inline]
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.advance_width, self.line_height)
    }

    #[inline]
    pub fn max_x(&self) -> f32 {
        self.pos.x + self.advance_width
    }

    pub fn logical_rect(&self) -> Rect {
        Rect::from_min_size(
            self.pos - vec2(0.0, self.font_ascent),
            vec2(self.advance_width, self.font_height),
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Row {
    /// This is included in case there are no glyphs.
    ///
    /// Only used during layout, then set to an invalid value in order to
    /// enable the paragraph-concat optimization path without having to
    /// adjust `section_index` when concatting.
    pub(crate) section_index_at_start: u32,

    /// One for each `char`.
    pub glyphs: Vec<Glyph>,

    /// Logical size based on font heights etc.
    /// Includes leading and trailing whitespace.
    pub size: Vec2,

    /// The mesh, ready to be rendered.
    pub visuals: RowVisuals,

    /// If true, this [`Row`] came from a paragraph ending with a `\n`.
    /// The `\n` itself is omitted from [`Self::glyphs`].
    /// A `\n` in the input text always creates a new [`Row`] below it,
    /// so that text that ends with `\n` has an empty [`Row`] last.
    /// This also implies that the last [`Row`] in a [`Galley`] always has `ends_with_newline == false`.
    pub ends_with_newline: bool,
}

/// The tessellated output of a row.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RowVisuals {
    /// The tessellated text, using non-normalized (texel) UV coordinates.
    /// That is, you need to divide the uv coordinates by the texture size.
    pub mesh: Mesh,

    /// Bounds of the mesh, and can be used for culling.
    /// Does NOT include leading or trailing whitespace glyphs!!
    pub mesh_bounds: Rect,

    /// The number of triangle indices added before the first glyph triangle.
    ///
    /// This can be used to insert more triangles after the background but before the glyphs,
    /// i.e. for text selection visualization.
    pub glyph_index_start: usize,

    /// The range of vertices in the mesh that contain glyphs (as opposed to background, underlines, strikethorugh, etc).
    ///
    /// The glyph vertices comes after backgrounds (if any), but before any underlines and strikethrough.
    pub glyph_vertex_range: Range<usize>,
}

impl Default for RowVisuals {
    fn default() -> Self {
        Self {
            mesh: Default::default(),
            mesh_bounds: Rect::NOTHING,
            glyph_index_start: 0,
            glyph_vertex_range: 0..0,
        }
    }
}

/// A single line of laid out text, ready for rendering
#[derive(Clone, Debug, PartialEq)]
pub struct LineGalley<'a> {
    /// The job that created this galley
    pub job: Arc<LineLayoutJob<'a>>,

    pub row_pos: Pos2,
    pub row: Row,

    /// Bounding rectangle
    pub rect: Rect,

    /// Mesh bounds for culling
    pub mesh_bounds: Rect,

    /// Rendering statistics
    pub num_vertices: usize,
    pub num_indices: usize,

    /// Pixels per point for the layout
    pub pixels_per_point: f32,
}

impl<'a> LineGalley<'a> {
    /// Check if the galley is empty
    pub fn is_empty(&self) -> bool {
        self.job.is_empty()
    }

    /// Get the size of the galley
    pub fn size(&self) -> Vec2 {
        self.rect.size()
    }

    /// Convert screen position to cursor position
    pub fn cursor_from_pos(&self, pos: Vec2) -> CCursor {
        let relative_x = pos.x - self.row_pos.x;
        let column = self.row.char_at(relative_x);

        // Convert screen column to rope column
        let rope_column = self.row.screen_col_to_rope_col(column);

        CCursor {
            index: rope_column,
            prefer_next_row: false,
        }
    }

    /// Convert cursor to screen position
    pub fn pos_from_cursor(&self, cursor: CCursor) -> Rect {
        // Convert rope column to screen column
        if let Some(screen_col) = self.row.rope_col_to_screen_col(cursor.index) {
            let x = self.row_pos.x + self.row.x_offset(screen_col);
            let y = self.row_pos.y;
            let height = self.row.size.y;
            Rect::from_min_max(pos2(x, y), pos2(x, y + height))
        } else {
            // Cursor is outside visible range, return end position
            let x = self.row_pos.x + self.row.size.x;
            let y = self.row_pos.y;
            let height = self.row.size.y;
            Rect::from_min_max(pos2(x, y), pos2(x, y + height))
        }
    }

    /// Move cursor left by one character
    pub fn cursor_left_one_character(&self, cursor: &CCursor) -> CCursor {
        CCursor {
            index: cursor.index.saturating_sub(1),
            prefer_next_row: false,
        }
    }

    /// Move cursor right by one character
    pub fn cursor_right_one_character(&self, cursor: &CCursor) -> CCursor {
        let max_index = self.job.rope.len_chars();
        CCursor {
            index: (cursor.index + 1).min(max_index),
            prefer_next_row: false,
        }
    }

    /// Get cursor at beginning of line
    pub fn cursor_begin_of_line(&self) -> CCursor {
        CCursor {
            index: 0,
            prefer_next_row: false,
        }
    }

    /// Get cursor at end of line
    pub fn cursor_end_of_line(&self) -> CCursor {
        CCursor {
            index: self.job.rope.len_chars(),
            prefer_next_row: false,
        }
    }
}

// Helper functions for creating common layouts
impl<'a> LineLayoutJob<'a> {}

