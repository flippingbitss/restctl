use std::{sync::Arc, usize};

use egui::{
    Color32, CornerRadius, CursorIcon, Event, EventFilter, Id, KeyboardShortcut, NumExt, Pos2,
    Rect, Sense, Stroke, Vec2,
    emath::TSTransform,
    response,
    scroll_area::ScrollSource,
    text::{CCursor, CCursorRange, LayoutJob},
};
use log::info;

use crate::code::{output::TextEditOutput, state::TextEditState, text_buffer::RopeBuffer};

type LayouterFn<'t> = &'t mut dyn FnMut(&egui::Ui, &RopeBuffer, f32) -> Arc<egui::Galley>;

pub struct TextEdit<'t> {
    text: &'t mut RopeBuffer,
    id: Option<egui::Id>,
    id_salt: Option<egui::Id>,
    event_filter: EventFilter,
    layouter: Option<LayouterFn<'t>>,
    font_selection: egui::FontSelection,
    background_color: Option<egui::Color32>,
    return_key: Option<egui::KeyboardShortcut>,
    scroll_start_row: usize,
}

impl TextEdit<'_> {
    pub fn load_state(ctx: &egui::Context, id: egui::Id) -> Option<TextEditState> {
        TextEditState::load(ctx, id)
    }

    pub fn store_state(ctx: &egui::Context, id: egui::Id, state: TextEditState) {
        state.store(ctx, id);
    }
}

impl<'t> TextEdit<'t> {
    pub fn new(text: &'t mut RopeBuffer) -> Self {
        Self {
            text,
            id: None,
            id_salt: None,
            layouter: None,
            font_selection: egui::TextStyle::Monospace.into(),
            background_color: None,
            return_key: Some(egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::Enter,
            )),
            event_filter: EventFilter {
                // moving the cursor is really important
                horizontal_arrows: true,
                vertical_arrows: true,
                tab: false, // tab is used to change focus, not to insert a tab character
                ..Default::default()
            },
            scroll_start_row: 0,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<TextEditOutput> {
        let bg_color = self
            .background_color
            .unwrap_or_else(|| ui.visuals().text_edit_bg_color());

        self.show_contents(ui);

        // todo show frame
        // todo show contents
        None
    }

    fn show_contents(self, ui: &mut egui::Ui) {
        let TextEdit {
            id,
            id_salt,
            event_filter,
            return_key,
            ..
        } = self;
        const MIN_WIDTH: f32 = 128.0;
        const MIN_HEIGHT: f32 = 128.0;
        const MARGIN: egui::Margin = egui::Margin::symmetric(4, 2);
        let margin_sum = MARGIN.sum();
        let font_id = self.font_selection.resolve(ui.style());
        let row_height = ui.fonts(|f| f.row_height(&font_id));
        let char_width = ui.fonts(|f| f.glyph_width(&font_id, 'W'));

        let available_width = (ui.available_width() - margin_sum.x).at_least(MIN_WIDTH); // sdjflasdjflksdklfjaslkfjl
        // let available_height = (ui.available_height() - margin_sum.x).at_least(MIN_HEIGHT);
        let available_height = 100.0;

        // bounds of the widget
        let top_left = ui.cursor().min;
        let global_outer_rect =
            Rect::from_min_size(top_left, Vec2::new(available_width, available_height));
        let outer_rect =
            Rect::from_min_max(Pos2::ZERO, Pos2::new(available_width, available_height));
        //
        // ui.painter().rect_stroke(
        //     global_outer_rect,
        //     CornerRadius::ZERO,
        //     Stroke::new(1.0, Color32::GREEN),
        //     egui::StrokeKind::Inside,
        // );
        //
        // let painter = ui.painter_at(global_outer_rect);
        // painter.rect_stroke(
        //     Rect::from_min_max(Pos2::ZERO, Pos2::new(500.0, 500.0)),
        //     CornerRadius::ZERO,
        //     Stroke::new(1.0, Color32::RED),
        //     egui::StrokeKind::Inside,
        // );

        // todo: use rope, string isn't gonna work for large bodies
        // but this is for MVP
        let total_chars = self.text.char_count();
        let total_rows = self.text.line_count();
        let id = ui.make_persistent_id("blah blah");
        //
        // let id = id.unwrap_or_else(|| {
        //     if let Some(id_salt) = id_salt {
        //         ui.make_persistent_id(id_salt)
        //     } else {
        //         ui.next_auto_id() // might need to change to this to be always persistent
        //     }
        // });
        //
        let mut state = TextEditState::load(ui.ctx(), id).unwrap_or_default();

        let paint_rows =
            |text: &mut RopeBuffer, ui: &mut egui::Ui, row_range: std::ops::Range<usize>| {
                let start = row_range.start;
                let end = row_range.end;
                // for row_idx in row_range {
                //     // log::info!("row_index {}", row_idx);
                //     let pos = Pos2::new(
                //         global_outer_rect.min.x,
                //         global_outer_rect.min.y + (row_idx - start) as f32 * row_height,
                //     );
                //     let job = LayoutJob::simple_singleline(
                //         rows[row_idx].clone(),
                //         font_id.clone(),
                //         Color32::WHITE,
                //     );
                //     let galley = ui.fonts(|f| f.layout_job(job));
                //     ui.painter().galley(pos, galley, Color32::RED);
                // }

                let scoped_text = text.char_range(start..end).to_string();
                let job = LayoutJob::simple(scoped_text, font_id, Color32::WHITE, available_width);
                let galley = ui.fonts(|f| f.layout_job(job));
                ui.painter()
                    .galley(global_outer_rect.min, galley, Color32::RED);
            };

        // log::info!("row_height {}", row_height);
        let output = egui::ScrollArea::new([true, true])
            .auto_shrink(false)
            .scroll_source(ScrollSource {
                scroll_bar: true,
                drag: false,
                mouse_wheel: true,
            })
            .min_scrolled_height(available_height / 2.0)
            .max_width(available_width)
            .max_height(available_height)
            .show_rows(ui, row_height, total_rows, |ui, row_range| {
                paint_rows(self.text, ui, row_range);
                //
                // let response =
                //     ui.interact(global_outer_rect, Id::new("testscroll"), Sense::click());
                // if let Some(pointer) = response.interact_pointer_pos() {
                //     info!("pointer found: {}", pointer);
                // }
                //
                // On touch screens (e.g. mobile in `eframe` web), should
                // dragging select text, or scroll the enclosing [`ScrollArea`] (if any)?
                // Since currently copying selected text in not supported on `eframe` web,
                // we prioritize touch-scrolling:
                let allow_drag_to_select =
                    ui.input(|i| !i.has_touch_screen()) || ui.memory(|mem| mem.has_focus(id));

                // skip interactive test
                let sense = if allow_drag_to_select {
                    Sense::click_and_drag()
                } else {
                    Sense::click()
                };

                let mut response = ui.interact(global_outer_rect, id, sense);
                // log::info!("outer_rect {}", outer_rect);
                // log::info!("global_outer_rect {}", global_outer_rect);
                // skip intrinsic_size

                ui.painter().rect_stroke(
                    response.rect,
                    CornerRadius::ZERO,
                    Stroke::new(1.0, Color32::GREEN),
                    egui::StrokeKind::Inside,
                );
                // Don't sent `OutputEvent::Clicked` when a user presses the space bar
                response.flags -= response::Flags::FAKE_PRIMARY_CLICKED;
                // skip clip text rect expansion

                // log::info!("response: {:?}", response);

                let painter = ui.painter_at(outer_rect);
                // get mouse/touch position if interacted
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    // skip mutable text test

                    if response.hovered() {
                        ui.output_mut(|o| o.mutable_text_under_cursor = true);
                    }

                    log::info!(
                        "pointer_pos {}, hovered : {}",
                        pointer_pos,
                        response.hovered()
                    );
                    let singleline_offset = Vec2::new(state.singleline_offset, 0.0);
                    let cursor_at_pointer = Default::default();
                    // let cursor_at_pointer = Self::find_cursor_from_pos(
                    //     pointer_pos - global_outer_rect.min + singleline_offset,
                    //     &total_rows,
                    //     &row_height,
                    //     &rows,
                    //     &total_chars,
                    //     &char_width,
                    //     &outer_rect.max.y,
                    // );
                    //
                    // log::info!("cursor_at_pointer {:?}", cursor_at_pointer);
                    log::info!("preview cursor {:?}", ui.visuals().text_cursor.preview);
                    log::info!("hovered {:?}", response.hovered());
                    log::info!(
                        "pointer is_moving {:?}",
                        ui.input(|i| i.pointer.is_moving())
                    );

                    // CURSOR PREVIEW - preview cursor while hovering but haven't clicked yet
                    if ui.visuals().text_cursor.preview
                        && response.hovered()
                        && ui.input(|i| i.pointer.is_moving())
                    {
                        // text cursor preview:
                        let cursor_rect =
                            TSTransform::from_translation(global_outer_rect.min.to_vec2())
                                * Rect::from_two_pos(
                                    Pos2::new(0.0, 5.0),
                                    Pos2::new(10.0, row_height * 1.15),
                                );

                        log::info!("cursor_rect: {:?}", cursor_rect);

                        // * cursor_rect(&galley, &cursor_at_pointer, row_height);
                        egui::text_selection::visuals::paint_cursor_end(
                            &painter,
                            ui.visuals(),
                            cursor_rect,
                        );
                    }

                    // TEXT SELECTION
                    // todo : re impl Galley's TextCursorState based on rope, to support selection
                    //
                    // let is_being_dragged = ui.ctx().is_being_dragged(response.id);
                    //
                    // let did_interact = state.cursor.pointer_interaction(
                    //     ui,
                    //     &response,
                    //     cursor_at_pointer,
                    //     &galley,
                    //     is_being_dragged,
                    // );
                    //
                    // if did_interact || response.clicked() {
                    //     ui.memory_mut(|mem| mem.request_focus(response.id));
                    //
                    //     state.last_interaction_time = ui.ctx().input(|i| i.time);
                    // }

                    if response.clicked() {
                        log::info!("focusing and cursor_at_pointer: {:?}", cursor_at_pointer);
                        state
                            .cursor
                            .set_char_range(Some(CCursorRange::one(cursor_at_pointer)));
                        ui.memory_mut(|mem| mem.request_focus(id));
                    }
                }

                // show text cursor when hovering
                if response.hovered() {
                    ui.ctx().set_cursor_icon(CursorIcon::Text);
                }

                let mut cursor_range = None;
                let prev_cursor_range = state.cursor.char_range(); // skip, this needs clamping

                if ui.memory(|mem| mem.has_focus(id)) {
                    ui.memory_mut(|mem| mem.set_focus_lock_filter(id, event_filter));

                    let default_cursor_range = CCursorRange::default();

                    let (changed, new_cursor_range) = events(
                        ui,
                        &mut state,
                        self.text,
                        id,
                        default_cursor_range,
                        event_filter,
                        return_key,
                    );

                    if changed {
                        response.mark_changed();
                    }
                    info!("changed: {}: {:?}", changed, new_cursor_range);
                    cursor_range = Some(new_cursor_range);
                }

                let selection_changed = if let (Some(cursor_range), Some(prev_cursor_range)) =
                    (cursor_range, prev_cursor_range)
                {
                    prev_cursor_range != cursor_range
                } else {
                    false
                };

                state.clone().store(ui.ctx(), id);
            });
    }

    fn find_cursor_from_pos(
        pos: Vec2,
        total_rows: &usize,
        row_height: &f32,
        rows: &Vec<String>,
        total_chars: &usize,
        char_width: &f32,
        outer_rect_max_y: &f32,
    ) -> CCursor {
        let VMARGIN = 5.0;
        // let min = outer_rect.min;
        // let min_y = min.y;
        let min_y = 0.0;
        let max_rect_y = *outer_rect_max_y;
        let max_rows_y = (*total_rows as f32 * *row_height);

        let max_y = f32::min(max_rect_y, max_rows_y);

        if rows.is_empty() || pos.y < min_y - VMARGIN {
            return CCursor::default();
        }

        if pos.y > max_y + VMARGIN {
            return CCursor {
                index: *total_chars,
                prefer_next_row: false,
            };
        }

        // need to consider row_range
        let row_index = (pos.y / *row_height).floor() as usize;
        let row = rows.get(row_index).expect("failed to get row");

        let base_index = rows
            .iter()
            .take(row_index)
            .map(|r| r.len() + 1)
            .sum::<usize>();

        let col_index = (pos.x / *char_width).floor() as usize;

        CCursor {
            index: base_index + col_index,
            prefer_next_row: col_index < row.len(),
        }
    }
}

/// Check for (keyboard) events to edit the cursor and/or text.
#[expect(clippy::too_many_arguments)]
fn events(
    ui: &egui::Ui,
    state: &mut TextEditState,
    text: &mut RopeBuffer,
    id: Id,
    default_cursor_range: CCursorRange,
    event_filter: EventFilter,
    return_key: Option<KeyboardShortcut>,
) -> (bool, CCursorRange) {
    let os = ui.ctx().os();

    let mut cursor_range = state.cursor.char_range().unwrap_or(default_cursor_range);

    // We feed state to the undoer both before and after handling input
    // so that the undoer creates automatic saves even when there are no events for a while.
    // state.undoer.lock().feed_state(
    //     ui.input(|i| i.time),
    //     &(cursor_range, text.as_str().to_owned()),
    // );

    let mut any_change = false;

    let mut events = ui.input(|i| i.filtered_events(&event_filter));

    for event in &events {
        let did_mutate_text = match event {
            // First handle events that only changes the selection cursor, not the text:
            // event if cursor_range.on_event(os, event, galley, id) => None,
            Event::Text(text_to_insert) => {
                // Newlines are handled by `Key::Enter`.
                if !text_to_insert.is_empty() && text_to_insert != "\n" && text_to_insert != "\r" {
                    let mut ccursor = text.delete_selected(&cursor_range);
                    text.insert_text(text_to_insert, ccursor.index);
                    Some(CCursorRange::one(ccursor))
                } else {
                    None
                }
            }
            // Event::Key {
            //     key: Key::Tab,
            //     pressed: true,
            //     modifiers,
            //     ..
            // } if multiline => {
            //     let mut ccursor = text.delete_selected(&cursor_range);
            //     if modifiers.shift {
            //         // TODO(emilk): support removing indentation over a selection?
            //         text.decrease_indentation(&mut ccursor);
            //     } else {
            //         text.insert_text_at(&mut ccursor, "\t", char_limit);
            //     }
            //     Some(CCursorRange::one(ccursor))
            // }
            // Event::Key {
            //     key,
            //     pressed: true,
            //     modifiers,
            //     ..
            // } if return_key.is_some_and(|return_key| {
            //     *key == return_key.logical_key && modifiers.matches_logically(return_key.modifiers)
            // }) =>
            // {
            //     if multiline {
            //         let mut ccursor = text.delete_selected(&cursor_range);
            //         text.insert_text_at(&mut ccursor, "\n", char_limit);
            //         // TODO(emilk): if code editor, auto-indent by same leading tabs, + one if the lines end on an opening bracket
            //         Some(CCursorRange::one(ccursor))
            //     } else {
            //         ui.memory_mut(|mem| mem.surrender_focus(id)); // End input with enter
            //         break;
            //     }
            // }
            //
            // Event::Key {
            //     key,
            //     pressed: true,
            //     modifiers,
            //     ..
            // } if (modifiers.matches_logically(Modifiers::COMMAND) && *key == Key::Y)
            //     || (modifiers.matches_logically(Modifiers::SHIFT | Modifiers::COMMAND)
            //         && *key == Key::Z) =>
            // {
            //     if let Some((redo_ccursor_range, redo_txt)) = state
            //         .undoer
            //         .lock()
            //         .redo(&(cursor_range, text.as_str().to_owned()))
            //     {
            //         text.replace_with(redo_txt);
            //         Some(*redo_ccursor_range)
            //     } else {
            //         None
            //     }
            // }
            //
            // Event::Key {
            //     key: Key::Z,
            //     pressed: true,
            //     modifiers,
            //     ..
            // } if modifiers.matches_logically(Modifiers::COMMAND) => {
            //     if let Some((undo_ccursor_range, undo_txt)) = state
            //         .undoer
            //         .lock()
            //         .undo(&(cursor_range, text.as_str().to_owned()))
            //     {
            //         text.replace_with(undo_txt);
            //         Some(*undo_ccursor_range)
            //     } else {
            //         None
            //     }
            // }
            //
            // Event::Key {
            //     modifiers,
            //     key,
            //     pressed: true,
            //     ..
            // } => check_for_mutating_key_press(os, &cursor_range, text, galley, modifiers, *key),
            _ => None,
        };

        if let Some(new_ccursor_range) = did_mutate_text {
            any_change = true;

            // Layout again to avoid frame delay, and to keep `text` and `galley` in sync.
            // *galley = layouter(ui, text, wrap_width);

            // Set cursor_range using new galley:
            cursor_range = new_ccursor_range;
        }
    }

    state.cursor.set_char_range(Some(cursor_range));

    // state.undoer.lock().feed_state(
    //     ui.input(|i| i.time),
    //     &(cursor_range, text.as_str().to_owned()),
    // );
    //
    (any_change, cursor_range)
}
