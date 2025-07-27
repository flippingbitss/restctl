use std::sync::Arc;

use egui::emath::{Align, GuiRounding as _, NumExt as _, Pos2, Rect, Vec2, pos2, vec2};

use egui::epaint::{PathStroke, Vertex};
use egui::{Color32, Mesh, Stroke};

use crate::code::text_layout_types::{
    Glyph, LineGalley, LineLayoutJob, LineSection, Row, RowVisuals,
};

// ----------------------------------------------------------------------------

/// Represents GUI scale and convenience methods for rounding to pixels.
#[derive(Clone, Copy)]
struct PointScale {
    pub pixels_per_point: f32,
}

impl PointScale {
    #[inline(always)]
    pub fn new(pixels_per_point: f32) -> Self {
        Self { pixels_per_point }
    }

    #[inline(always)]
    pub fn pixels_per_point(&self) -> f32 {
        self.pixels_per_point
    }

    #[inline(always)]
    pub fn round_to_pixel(&self, point: f32) -> f32 {
        (point * self.pixels_per_point).round() / self.pixels_per_point
    }

    #[inline(always)]
    pub fn floor_to_pixel(&self, point: f32) -> f32 {
        (point * self.pixels_per_point).floor() / self.pixels_per_point
    }
}

#[derive(Default)]
struct FormatSummary {
    any_background: bool,
    any_underline: bool,
    any_strikethrough: bool,
}

fn format_summary(job: &LineLayoutJob) -> FormatSummary {
    let mut format_summary = FormatSummary::default();
    for section in &job.sections {
        format_summary.any_background |= section.format.background_color != Color32::TRANSPARENT;
        format_summary.any_underline |= section.format.underline != Stroke::NONE;
        format_summary.any_strikethrough |= section.format.strikethrough != Stroke::NONE;
    }
    format_summary
}

/// Calculate the Y positions and tessellate the text.
fn galley_from_row<'a>(
    point_scale: PointScale,
    job: Arc<LineLayoutJob<'a>>,
    mut row: Row,
) -> LineGalley<'a> {
    let mut cursor_y = 0.0;
    let mut row_pos = Pos2::new(0.0, 0.0);

    let mut max_row_height = 0.0;
    if let Some(glyph) = row.glyphs.first() {
        max_row_height = max_row_height.at_least(glyph.font_height);
    }
    max_row_height = point_scale.round_to_pixel(max_row_height);

    // Now position each glyph vertically:
    for glyph in &mut row.glyphs {
        // todo: calculate y
        glyph.pos.y = glyph.font_ascent;
        glyph.pos.y = point_scale.round_to_pixel(glyph.pos.y);
    }

    row_pos.y = cursor_y;
    row.size.y = max_row_height;

    cursor_y += max_row_height;
    cursor_y = point_scale.round_to_pixel(cursor_y); // TODO(emilk): it would be better to do the calculations in pixels instead.

    let format_summary = format_summary(&job);

    row.visuals = tessellate_row(point_scale, &job, format_summary, &row);
    let rect = Rect::from_min_size(row_pos, row.size);
    let mesh_bounds = row.visuals.mesh_bounds;
    let num_vertices = row.visuals.mesh.vertices.len();
    let num_indices = row.visuals.mesh.indices.len();

    let mut galley = LineGalley {
        job,
        row_pos,
        row: row,
        rect,
        mesh_bounds,
        num_vertices,
        num_indices,
        pixels_per_point: point_scale.pixels_per_point,
    };

    galley
}

fn tessellate_row<'a>(
    point_scale: PointScale,
    job: &LineLayoutJob<'a>,
    format_summary: FormatSummary,
    row: &Row,
) -> RowVisuals {
    if row.glyphs.is_empty() {
        return Default::default();
    }

    let mut mesh = Mesh::default();

    mesh.reserve_triangles(row.glyphs.len() * 2);
    mesh.reserve_vertices(row.glyphs.len() * 4);

    if format_summary.any_background {
        add_row_backgrounds(point_scale, job, row, &mut mesh);
    }

    let glyph_index_start = mesh.indices.len();
    let glyph_vertex_start = mesh.vertices.len();
    tessellate_glyphs(point_scale, job, row, &mut mesh);
    let glyph_vertex_end = mesh.vertices.len();
    //
    // if format_summary.any_underline {
    //     add_row_hline(point_scale, row, &mut mesh, |glyph| {
    //         let format = &job.sections[glyph.section_index as usize].format;
    //         let stroke = format.underline;
    //         let y = glyph.logical_rect().bottom();
    //         (stroke, y)
    //     });
    // }
    //
    // if format_summary.any_strikethrough {
    //     add_row_hline(point_scale, row, &mut mesh, |glyph| {
    //         let format = &job.sections[glyph.section_index as usize].format;
    //         let stroke = format.strikethrough;
    //         let y = glyph.logical_rect().center().y;
    //         (stroke, y)
    //     });
    // }
    //
    let mesh_bounds = mesh.calc_bounds();

    RowVisuals {
        mesh,
        mesh_bounds,
        glyph_index_start,
        glyph_vertex_range: glyph_vertex_start..glyph_vertex_end,
    }
}

/// Create background for glyphs that have them.
/// Creates as few rectangular regions as possible.
fn add_row_backgrounds<'a>(
    point_scale: PointScale,
    job: &LineLayoutJob<'a>,
    row: &Row,
    mesh: &mut Mesh,
) {
    if row.glyphs.is_empty() {
        return;
    }

    let mut end_run = |start: Option<(Color32, Rect, f32)>, stop_x: f32| {
        if let Some((color, start_rect, expand)) = start {
            let rect = Rect::from_min_max(start_rect.left_top(), pos2(stop_x, start_rect.bottom()));
            let rect = rect.expand(expand);
            let rect = rect.round_to_pixels(point_scale.pixels_per_point());
            mesh.add_colored_rect(rect, color);
        }
    };

    let mut run_start = None;
    let mut last_rect = Rect::NAN;

    for glyph in &row.glyphs {
        let format = &job.sections[glyph.run_index as usize].format;
        let color = format.background_color;
        let rect = glyph.logical_rect();

        if color == Color32::TRANSPARENT {
            end_run(run_start.take(), last_rect.right());
        } else if let Some((existing_color, start, expand)) = run_start {
            if existing_color == color
                && start.top() == rect.top()
                && start.bottom() == rect.bottom()
                && format.expand_bg == expand
            {
                // continue the same background rectangle
            } else {
                end_run(run_start.take(), last_rect.right());
                run_start = Some((color, rect, format.expand_bg));
            }
        } else {
            run_start = Some((color, rect, format.expand_bg));
        }

        last_rect = rect;
    }

    end_run(run_start.take(), last_rect.right());
}

fn tessellate_glyphs<'a>(
    point_scale: PointScale,
    job: &LineLayoutJob<'a>,
    row: &Row,
    mesh: &mut Mesh,
) {
    for glyph in &row.glyphs {
        let uv_rect = glyph.uv_rect;
        if !uv_rect.is_nothing() {
            let mut left_top = glyph.pos + uv_rect.offset;
            left_top.x = point_scale.round_to_pixel(left_top.x);
            left_top.y = point_scale.round_to_pixel(left_top.y);

            let rect = Rect::from_min_max(left_top, left_top + uv_rect.size);
            let uv = Rect::from_min_max(
                pos2(uv_rect.min[0] as f32, uv_rect.min[1] as f32),
                pos2(uv_rect.max[0] as f32, uv_rect.max[1] as f32),
            );

            let format = &job.sections[glyph.run_index as usize].format;

            let color = format.color;

            if format.italics {
                let idx = mesh.vertices.len() as u32;
                mesh.add_triangle(idx, idx + 1, idx + 2);
                mesh.add_triangle(idx + 2, idx + 1, idx + 3);

                let top_offset = rect.height() * 0.25 * Vec2::X;

                mesh.vertices.push(Vertex {
                    pos: rect.left_top() + top_offset,
                    uv: uv.left_top(),
                    color,
                });
                mesh.vertices.push(Vertex {
                    pos: rect.right_top() + top_offset,
                    uv: uv.right_top(),
                    color,
                });
                mesh.vertices.push(Vertex {
                    pos: rect.left_bottom(),
                    uv: uv.left_bottom(),
                    color,
                });
                mesh.vertices.push(Vertex {
                    pos: rect.right_bottom(),
                    uv: uv.right_bottom(),
                    color,
                });
            } else {
                mesh.add_rect_with_uv(rect, uv, color);
            }
        }
    }
}
