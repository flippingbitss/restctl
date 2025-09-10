use std::{sync::Arc, usize};

use egui::{
    CornerRadius, Layout, RectAlign, Stroke,
    emath::{Rect, TSTransform},
};
use epaint::{
    StrokeKind,
    text::{Galley, LayoutJob, cursor::CCursor},
};

use egui::{
    Align, Align2, Color32, Context, CursorIcon, Event, EventFilter, FontSelection, Id, ImeEvent,
    Key, KeyboardShortcut, Margin, Modifiers, NumExt as _, Response, Sense, Shape, TextBuffer,
    TextStyle, TextWrapMode, Ui, Vec2, Widget, WidgetInfo, WidgetText, WidgetWithState, epaint,
    os::OperatingSystem,
    output::OutputEvent,
    response, text_selection,
    text_selection::{CCursorRange, text_cursor_state::cursor_rect, visuals::paint_text_selection},
    vec2,
};
use tree_sitter::{Language, ParseOptions, Point, ffi::TSInput};

use crate::code::{
    autocomplete::autocomplete_at_cursor, output::TextEditOutput, state::TextEditState,
};

type LayouterFn<'t> = &'t mut dyn FnMut(&Ui, &dyn TextBuffer, f32) -> Arc<Galley>;

/// A text region that the user can edit the contents of.
///
/// See also [`Ui::text_edit_singleline`] and [`Ui::text_edit_multiline`].
///
/// Example:
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// # let mut my_string = String::new();
/// let response = ui.add(egui::TextEdit::singleline(&mut my_string));
/// if response.changed() {
///     // …
/// }
/// if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
///     // …
/// }
/// # });
/// ```
///
/// To fill an [`Ui`] with a [`TextEdit`] use [`Ui::add_sized`]:
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// # let mut my_string = String::new();
/// ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut my_string));
/// # });
/// ```
///
///
/// You can also use [`TextEdit`] to show text that can be selected, but not edited.
/// To do so, pass in a `&mut` reference to a `&str`, for instance:
///
/// ```
/// fn selectable_text(ui: &mut egui::Ui, mut text: &str) {
///     ui.add(egui::TextEdit::multiline(&mut text));
/// }
/// ```
///
/// ## Advanced usage
/// See [`TextEdit::show`].
///
/// ## Other
/// The background color of a [`crate::TextEdit`] is [`crate::Visuals::text_edit_bg_color`] or can be set with [`crate::TextEdit::background_color`].
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct TextEdit<'t> {
    id: Option<Id>,
    id_salt: Option<Id>,
    // ----- parsing and text buffer
    text: &'t mut dyn TextBuffer,
    parser: &'t mut tree_sitter::Parser,
    tree: &'t mut tree_sitter::Tree,
    // ----- for autocompletion based on treesitter
    autocompletion: &'t mut AutoCompletionUiState,
    // ----- sizes and appearance
    font_selection: FontSelection,
    layouter: Option<LayouterFn<'t>>,
    frame: bool,
    desired_width: Option<f32>,
    desired_height_rows: usize,
    event_filter: EventFilter,
    return_key: Option<KeyboardShortcut>,
}

#[derive(Default)]
pub struct AutoCompletionUiState {
    selected_index: usize,
    debug_info: String,
    requested: bool,
    items: Vec<String>,
}

pub struct JsonSource {
    parser: tree_sitter::Parser,
    tree: tree_sitter::Tree,
    buffer: String,
    dirty: bool,
    incremental: bool,
}

impl JsonSource {
    pub fn new() -> Self {
        Self::text(String::new())
    }

    pub fn text(buffer: String) -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_json::LANGUAGE.into());
        let tree = parser.parse(&buffer, None).unwrap();
        Self {
            parser,
            tree,
            buffer,
            dirty: false,
            incremental: true,
        }
    }
}

impl WidgetWithState for TextEdit<'_> {
    type State = TextEditState;
}

impl TextEdit<'_> {
    pub fn load_state(ctx: &Context, id: Id) -> Option<TextEditState> {
        TextEditState::load(ctx, id)
    }

    pub fn store_state(ctx: &Context, id: Id, state: TextEditState) {
        state.store(ctx, id);
    }
}

impl<'t> TextEdit<'t> {
    /// A [`TextEdit`] for multiple lines. Pressing enter key will create a new line by default (can be changed with [`return_key`](TextEdit::return_key)).
    pub fn json(source: &'t mut JsonSource, autocompletion: &'t mut AutoCompletionUiState) -> Self {
        // // We only support JSON btw because that should be enough, right ? right ?
        // parser.set_language(&tree_sitter_json::LANGUAGE.into());
        Self {
            text: &mut source.buffer,
            parser: &mut source.parser,
            tree: &mut source.tree,
            autocompletion: autocompletion,
            id: None,
            id_salt: None,
            font_selection: TextStyle::Monospace.into(),
            layouter: None,
            frame: true,
            desired_width: None,
            desired_height_rows: 4,
            event_filter: EventFilter {
                // moving the cursor is really important
                horizontal_arrows: true,
                vertical_arrows: true,
                tab: false, // tab is used to change focus, not to insert a tab character
                ..Default::default()
            },
            return_key: Some(KeyboardShortcut::new(Modifiers::NONE, Key::Enter)),
        }
    }
    /// Use if you want to set an explicit [`Id`] for this widget.
    #[inline]
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// A source for the unique [`Id`], e.g. `.id_source("second_text_edit_field")` or `.id_source(loop_index)`.
    #[inline]
    pub fn id_source(self, id_salt: impl std::hash::Hash) -> Self {
        self.id_salt(id_salt)
    }

    /// A source for the unique [`Id`], e.g. `.id_salt("second_text_edit_field")` or `.id_salt(loop_index)`.
    #[inline]
    pub fn id_salt(mut self, id_salt: impl std::hash::Hash) -> Self {
        self.id_salt = Some(Id::new(id_salt));
        self
    }
    /// Pick a [`crate::FontId`] or [`TextStyle`].
    #[inline]
    pub fn font(mut self, font_selection: impl Into<FontSelection>) -> Self {
        self.font_selection = font_selection.into();
        self
    }
    /// Override how text is being shown inside the [`TextEdit`].
    ///
    /// This can be used to implement things like syntax highlighting.
    ///
    /// This function will be called at least once per frame,
    /// so it is strongly suggested that you cache the results of any syntax highlighter
    /// so as not to waste CPU highlighting the same string every frame.
    ///
    /// The arguments is the enclosing [`Ui`] (so you can access e.g. [`Ui::fonts`]),
    /// the text and the wrap width.
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// # let mut my_code = String::new();
    /// # fn my_memoized_highlighter(s: &str) -> egui::text::LayoutJob { Default::default() }
    /// let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
    ///     let mut layout_job: egui::text::LayoutJob = my_memoized_highlighter(buf.as_str());
    ///     layout_job.wrap.max_width = wrap_width;
    ///     ui.fonts(|f| f.layout_job(layout_job))
    /// };
    /// ui.add(egui::TextEdit::multiline(&mut my_code).layouter(&mut layouter));
    /// # });
    /// ```
    #[inline]
    pub fn layouter(
        mut self,
        layouter: &'t mut dyn FnMut(&Ui, &dyn TextBuffer, f32) -> Arc<Galley>,
    ) -> Self {
        self.layouter = Some(layouter);

        self
    }

    /// Default is `true`. If set to `false` there will be no frame showing that this is editable text!
    #[inline]
    pub fn frame(mut self, frame: bool) -> Self {
        self.frame = frame;
        self
    }

    /// Set to 0.0 to keep as small as possible.
    /// Set to [`f32::INFINITY`] to take up all available space (i.e. disable automatic word wrap).
    #[inline]
    pub fn desired_width(mut self, desired_width: f32) -> Self {
        self.desired_width = Some(desired_width);
        self
    }

    /// Set the number of rows to show by default.
    /// The default for singleline text is `1`.
    /// The default for multiline text is `4`.
    #[inline]
    pub fn desired_rows(mut self, desired_height_rows: usize) -> Self {
        self.desired_height_rows = desired_height_rows;
        self
    }

    /// When `false` (default), pressing TAB will move focus
    /// to the next widget.
    ///
    /// When `true`, the widget will keep the focus and pressing TAB
    /// will insert the `'\t'` character.
    #[inline]
    pub fn lock_focus(mut self, tab_will_indent: bool) -> Self {
        self.event_filter.tab = tab_will_indent;
        self
    }

    /// Set the return key combination.
    ///
    /// This combination will cause a newline on multiline,
    /// whereas on singleline it will cause the widget to lose focus.
    ///
    /// This combination is optional and can be disabled by passing [`None`] into this function.
    #[inline]
    pub fn return_key(mut self, return_key: impl Into<Option<KeyboardShortcut>>) -> Self {
        self.return_key = return_key.into();
        self
    }
}

// ----------------------------------------------------------------------------

impl Widget for TextEdit<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

impl TextEdit<'_> {
    /// Show the [`TextEdit`], returning a rich [`TextEditOutput`].
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// # let mut my_string = String::new();
    /// let output = egui::TextEdit::singleline(&mut my_string).show(ui);
    /// if let Some(text_cursor_range) = output.cursor_range {
    ///     use egui::TextBuffer as _;
    ///     let selected_chars = text_cursor_range.as_sorted_char_range();
    ///     let selected_text = my_string.char_range(selected_chars);
    ///     ui.label("Selected text: ");
    ///     ui.monospace(selected_text);
    /// }
    /// # });
    /// ```
    pub fn show(self, ui: &mut Ui) -> TextEditOutput {
        let is_mutable = self.text.is_mutable();
        let frame = self.frame;
        let where_to_put_background = ui.painter().add(Shape::Noop);
        let background_color = ui.visuals().text_edit_bg_color();

        fn print_node<'a>(
            result: &mut String,
            node: tree_sitter::Node<'a>,
            source: &str,
            indent: usize,
        ) {
            let kind = node.kind();
            let text = &source[node.byte_range()];
            result.push_str(&format!("{}{kind} ({:?})\n", "  ".repeat(indent), text));

            for i in 0..node.child_count() {
                let child = node.child(i).unwrap();
                print_node(result, child, source, indent + 1);
            }
        }

        let mut tree_str = String::new();
        print_node(&mut tree_str, self.tree.root_node(), self.text.as_str(), 0);

        // ui.label(tree_str);
        ui.label(self.tree.root_node().to_sexp());
        ui.separator();

        ui.label(format!("Debug Info: {}", self.autocompletion.debug_info));
        ui.separator();

        let output = self.show_content(ui);

        if frame {
            let visuals = ui.style().interact(&output.response);
            let frame_rect = output.response.rect.expand(visuals.expansion);
            let shape = if is_mutable {
                if output.response.has_focus() {
                    epaint::RectShape::new(
                        frame_rect,
                        visuals.corner_radius,
                        background_color,
                        ui.visuals().selection.stroke,
                        StrokeKind::Inside,
                    )
                } else {
                    epaint::RectShape::new(
                        frame_rect,
                        visuals.corner_radius,
                        background_color,
                        visuals.bg_stroke, // TODO(emilk): we want to show something here, or a text-edit field doesn't "pop".
                        StrokeKind::Inside,
                    )
                }
            } else {
                let visuals = &ui.style().visuals.widgets.inactive;
                epaint::RectShape::stroke(
                    frame_rect,
                    visuals.corner_radius,
                    visuals.bg_stroke, // TODO(emilk): we want to show something here, or a text-edit field doesn't "pop".
                    StrokeKind::Inside,
                )
            };

            ui.painter().set(where_to_put_background, shape);
        }

        output
    }

    fn show_content(self, ui: &mut Ui) -> TextEditOutput {
        let TextEdit {
            id,
            id_salt,
            text,
            parser,
            tree,
            font_selection,
            layouter,
            frame: _,
            desired_width,
            desired_height_rows,
            event_filter,
            return_key,
            autocompletion,
        } = self;

        let text_color = ui
            .visuals()
            .override_text_color
            .unwrap_or_else(|| ui.visuals().widgets.inactive.text_color());

        let prev_text = text.as_str().to_owned();

        let font_id = font_selection.resolve(ui.style());
        let row_height = ui.fonts(|f| f.row_height(&font_id));
        let margin = Margin::same(4);
        const MIN_WIDTH: f32 = 24.0; // Never make a [`TextEdit`] more narrow than this.
        let available_width = (ui.available_width() - margin.sum().x).at_least(MIN_WIDTH);
        let desired_width = desired_width.unwrap_or_else(|| ui.spacing().text_edit_width);
        let wrap_width = if ui.layout().horizontal_justify() {
            available_width
        } else {
            desired_width.min(available_width)
        };

        let font_id_clone = font_id.clone();
        let mut default_layouter = move |ui: &Ui, text: &dyn TextBuffer, wrap_width: f32| {
            let text = text.as_str().to_owned();
            let layout_job = LayoutJob::simple(text, font_id_clone.clone(), text_color, wrap_width);
            ui.fonts(|f| f.layout_job(layout_job))
        };

        let layouter = layouter.unwrap_or(&mut default_layouter);

        let mut galley = layouter(ui, text, wrap_width);

        let desired_inner_width = galley.size().x.max(wrap_width);
        let desired_height = (desired_height_rows.at_least(1) as f32) * row_height;
        let desired_inner_size = vec2(desired_inner_width, galley.size().y.max(desired_height));
        let desired_outer_size = (desired_inner_size + margin.sum());
        let (auto_id, outer_rect) = ui.allocate_space(desired_outer_size);
        let rect = outer_rect - margin; // inner rect (excluding frame/margin).

        let id = id.unwrap_or_else(|| {
            if let Some(id_salt) = id_salt {
                ui.make_persistent_id(id_salt)
            } else {
                auto_id // Since we are only storing the cursor a persistent Id is not super important
            }
        });
        let mut state = TextEditState::load(ui.ctx(), id).unwrap_or_default();

        // On touch screens (e.g. mobile in `eframe` web), should
        // dragging select text, or scroll the enclosing [`ScrollArea`] (if any)?
        // Since currently copying selected text in not supported on `eframe` web,
        // we prioritize touch-scrolling:
        let allow_drag_to_select =
            ui.input(|i| !i.has_touch_screen()) || ui.memory(|mem| mem.has_focus(id));

        let sense = if allow_drag_to_select {
            Sense::click_and_drag()
        } else {
            Sense::click()
        };

        let mut response = ui.interact(outer_rect, id, sense);
        response.intrinsic_size = Some(Vec2::new(desired_width, desired_outer_size.y));

        // log::info!("text_edit response {:?}", response);

        ui.painter().rect_stroke(
            response.rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, Color32::BLUE),
            egui::StrokeKind::Inside,
        );

        // Don't sent `OutputEvent::Clicked` when a user presses the space bar
        response.flags -= response::Flags::FAKE_PRIMARY_CLICKED;
        let text_clip_rect = rect;
        let painter = ui.painter_at(text_clip_rect.expand(1.0)); // expand to avoid clipping cursor

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            if response.hovered() && text.is_mutable() {
                ui.output_mut(|o| o.mutable_text_under_cursor = true);
            }

            // TODO(emilk): drag selected text to either move or clone (ctrl on windows, alt on mac)

            let singleline_offset = vec2(state.singleline_offset, 0.0);
            let cursor_at_pointer =
                galley.cursor_from_pos(pointer_pos - rect.min + singleline_offset);

            if ui.visuals().text_cursor.preview
                && response.hovered()
                && ui.input(|i| i.pointer.is_moving())
            {
                // text cursor preview:
                let cursor_rect = TSTransform::from_translation(rect.min.to_vec2())
                    * cursor_rect(&galley, &cursor_at_pointer, row_height);
                text_selection::visuals::paint_cursor_end(&painter, ui.visuals(), cursor_rect);
            }

            let is_being_dragged = ui.ctx().is_being_dragged(response.id);
            let did_interact = state.cursor.pointer_interaction(
                ui,
                &response,
                cursor_at_pointer,
                &galley,
                is_being_dragged,
            );

            if did_interact || response.clicked() {
                ui.memory_mut(|mem| mem.request_focus(response.id));

                state.last_interaction_time = ui.ctx().input(|i| i.time);
            }
        }

        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::Text);
        }

        let mut cursor_range = None;
        let prev_cursor_range = state.cursor.range(&galley);
        if ui.memory(|mem| mem.has_focus(id)) {
            ui.memory_mut(|mem| mem.set_focus_lock_filter(id, event_filter));
            let default_cursor_range = CCursorRange::default();

            let (changed, new_cursor_range) = events(
                ui,
                &mut state,
                text,
                &mut galley,
                layouter,
                autocompletion,
                id,
                wrap_width,
                default_cursor_range,
                event_filter,
                return_key,
            );

            if changed {
                response.mark_changed();

                // TODO: parse incrementally by applying edits to the old tree instead
                // for MVP, we are parsing from scratch on every event
                if let Some(new_tree) = parser.parse(text.as_str(), None) {
                    *tree = new_tree;
                }

                // self.tree = self.parser.parse(self.text);
            }
            cursor_range = Some(new_cursor_range);
        }

        let mut galley_pos = rect.left_top();

        let selection_changed = if let (Some(cursor_range), Some(prev_cursor_range)) =
            (cursor_range, prev_cursor_range)
        {
            prev_cursor_range != cursor_range
        } else {
            false
        };

        if autocompletion.requested {
            let cursor = cursor_range.and_then(|cr| cr.single());
            // Only if we are typing we show completions,
            // not during selections, cuts/deletes etc
            if let Some(cursor) = cursor {
                let coord = galley.layout_from_cursor(cursor);
                let mut debug_info = String::new();
                debug_info.push_str(&format!(
                    "Cursor row: {}, column: {}\n",
                    coord.row, coord.column
                ));

                let root = tree.root_node();
                let cursor_location = Point::new(coord.row, coord.column);
                let text_buffer = text.as_str();

                if let Some(node) =
                    root.named_descendant_for_point_range(cursor_location, cursor_location)
                {
                    debug_info.push_str(&format!("Active node: {}\n", node.kind()));
                    let slice = &text_buffer[node.byte_range()];
                    debug_info.push_str(&format!("Active range value: {}\n", slice));
                }
                // get 2d cursor
                // get active node
                // get active json key from node range offset and galley
                let (info_str, items) =
                    autocomplete_at_cursor(text_buffer, root, cursor.index, cursor_location);
                debug_info.push_str(&format!("Result from string like node: {}\n", info_str));

                autocompletion.items = items;
                autocompletion.debug_info = debug_info;
            }
        }

        if ui.is_rect_visible(rect) {
            let has_focus = ui.memory(|mem| mem.has_focus(id));

            if has_focus {
                if let Some(cursor_range) = state.cursor.range(&galley) {
                    // Add text selection rectangles to the galley:
                    paint_text_selection(&mut galley, ui.visuals(), &cursor_range, None);
                }
            }

            // if !clip_text {
            // Allocate additional space if edits were made this frame that changed the size. This is important so that,
            // if there's a ScrollArea, it can properly scroll to the cursor.
            // Condition `!clip_text` is important to avoid breaking layout for `TextEdit::singleline` (PR #5640)
            let extra_size = galley.size() - rect.size();
            if extra_size.x > 0.0 || extra_size.y > 0.0 {
                ui.allocate_rect(
                    Rect::from_min_size(outer_rect.max, extra_size),
                    Sense::hover(),
                );
            }
            // }

            painter.galley(galley_pos, galley.clone(), text_color);

            if has_focus {
                if let Some(cursor_range) = state.cursor.range(&galley) {
                    let primary_cursor_rect =
                        cursor_rect(&galley, &cursor_range.primary, row_height)
                            .translate(galley_pos.to_vec2());

                    if response.changed() || selection_changed {
                        // Scroll to keep primary cursor in view:
                        ui.scroll_to_rect(primary_cursor_rect + margin, None);
                    }

                    if text.is_mutable() {
                        let now = ui.ctx().input(|i| i.time);
                        if response.changed() || selection_changed {
                            state.last_interaction_time = now;
                        }

                        // Only show (and blink) cursor if the egui viewport has focus.
                        // This is for two reasons:
                        // * Don't give the impression that the user can type into a window without focus
                        // * Don't repaint the ui because of a blinking cursor in an app that is not in focus
                        let viewport_has_focus = ui.ctx().input(|i| i.focused);
                        if viewport_has_focus {
                            text_selection::visuals::paint_text_cursor(
                                ui,
                                &painter,
                                primary_cursor_rect,
                                now - state.last_interaction_time,
                            );
                        }

                        // Set IME output (in screen coords) when text is editable and visible
                        let to_global = ui
                            .ctx()
                            .layer_transform_to_global(ui.layer_id())
                            .unwrap_or_default();

                        ui.ctx().output_mut(|o| {
                            o.ime = Some(egui::output::IMEOutput {
                                rect: to_global * rect,
                                cursor_rect: to_global * primary_cursor_rect,
                            });
                        });
                    }

                    // Draw autocomplete UI
                    show_autocomplete_menu(ui, primary_cursor_rect, autocompletion);
                }
            }
        }
        //
        // // Ensures correct IME behavior when the text input area gains or loses focus.
        // if state.ime_enabled && (response.gained_focus() || response.lost_focus()) {
        //     state.ime_enabled = false;
        //     if let Some(mut ccursor_range) = state.cursor.char_range() {
        //         ccursor_range.secondary.index = ccursor_range.primary.index;
        //         state.cursor.set_char_range(Some(ccursor_range));
        //     }
        //     ui.input_mut(|i| i.events.retain(|e| !matches!(e, Event::Ime(_))));
        // }
        //
        state.clone().store(ui.ctx(), id);
        //
        // if response.changed() {
        //     response.widget_info(|| {
        //         WidgetInfo::text_edit(
        //             ui.is_enabled(),
        //             mask_if_password(password, prev_text.as_str()),
        //             mask_if_password(password, text.as_str()),
        //             hint_text_str.as_str(),
        //         )
        //     });
        // } else if selection_changed {
        //     let cursor_range = cursor_range.unwrap();
        //     let char_range = cursor_range.primary.index..=cursor_range.secondary.index;
        //     let info = WidgetInfo::text_selection_changed(
        //         ui.is_enabled(),
        //         char_range,
        //         mask_if_password(password, text.as_str()),
        //     );
        //     response.output_event(OutputEvent::TextSelectionChanged(info));
        // } else {
        //     response.widget_info(|| {
        //         WidgetInfo::text_edit(
        //             ui.is_enabled(),
        //             mask_if_password(password, prev_text.as_str()),
        //             mask_if_password(password, text.as_str()),
        //             hint_text_str.as_str(),
        //         )
        //     });
        // }
        //
        TextEditOutput {
            response,
            galley,
            galley_pos,
            text_clip_rect,
            state,
            cursor_range,
        }
    }
}

fn show_autocomplete_menu(
    ui: &mut Ui,
    cursor_rect: Rect,
    autocompletion: &mut AutoCompletionUiState,
) {
    if autocompletion.requested && !autocompletion.items.is_empty() {
        egui::Popup::new(
            "autocompletion_popup".into(),
            ui.ctx().clone(),
            cursor_rect,
            ui.layer_id(),
        )
        .kind(egui::PopupKind::Menu)
        .align(RectAlign::BOTTOM_START)
        .layout(Layout::top_down_justified(Align::Min))
        .width(200.0)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
        .show(|ui| {
            for (index, item) in autocompletion.items.iter_mut().enumerate() {
                if ui
                    .selectable_value(&mut autocompletion.selected_index, index, item.as_str())
                    .clicked()
                {
                    autocompletion.requested = false;
                }
            }
        });
    }
}

// [Str("abc"), Str("def")]
//
// enum JsonValue {
//     Null,
//     Num(i32),
//     Object(Box<JsonValue>),
//     Array(Vec<JsonValue>),
// }
//
// struct JsonKey {
//     name: String,
//     values: Vec<JsonValue>,
// }
//
fn create_dummy_graph() {}

fn compute_path_to_active_string<'a>(
    root: tree_sitter::Node<'a>,
    text_buffer: &str,
    byte_offset: usize,
    cursor_col: usize,
    cursor_row: usize,
) -> Option<Vec<String>> {
    let cursor_point = Point::new(cursor_row, cursor_col);
    let active_node = root.named_descendant_for_point_range(cursor_point, cursor_point);
    if let Some(node) = active_node {
        if node.kind() == "string" {
            // within quotes now
            let prefix = node
                .child_by_field_name("string_content")
                .map(|contents_node| &text_buffer[contents_node.byte_range()])
                .unwrap_or("");

            let mut path = Vec::new();

            // figure out path for this node
            let mut current_node = node.parent();
            while let Some(current) = current_node {
                path.push(current.kind().to_owned());
                current_node = current.parent();
            }

            path.reverse();

            return Some(path);
        }
    }

    None
}

fn mask_if_password(is_password: bool, text: &str) -> String {
    fn mask_password(text: &str) -> String {
        std::iter::repeat_n(
            epaint::text::PASSWORD_REPLACEMENT_CHAR,
            text.chars().count(),
        )
        .collect::<String>()
    }

    if is_password {
        mask_password(text)
    } else {
        text.to_owned()
    }
}

// ----------------------------------------------------------------------------

/// Check for (keyboard) events to edit the cursor and/or text.
#[expect(clippy::too_many_arguments)]
fn events(
    ui: &egui::Ui,
    state: &mut TextEditState,
    text: &mut dyn TextBuffer,
    galley: &mut Arc<Galley>,
    layouter: &mut dyn FnMut(&Ui, &dyn TextBuffer, f32) -> Arc<Galley>,
    autocompletion: &mut AutoCompletionUiState,
    id: Id,
    wrap_width: f32,
    default_cursor_range: CCursorRange,
    event_filter: EventFilter,
    return_key: Option<KeyboardShortcut>,
) -> (bool, CCursorRange) {
    let os = ui.ctx().os();

    let mut cursor_range = state.cursor.range(galley).unwrap_or(default_cursor_range);

    // We feed state to the undoer both before and after handling input
    // so that the undoer creates automatic saves even when there are no events for a while.
    state.undoer.lock().feed_state(
        ui.input(|i| i.time),
        &(cursor_range, text.as_str().to_owned()),
    );

    let mut any_change = false;

    let mut events = ui.input(|i| i.filtered_events(&event_filter));

    for event in &events {
        let did_mutate_text = match event {
            // First handle events that only changes the selection cursor, not the text:
            event if cursor_range.on_event(os, event, galley, id) => None,

            Event::Copy => {
                if cursor_range.is_empty() {
                    None
                } else {
                    let text = cursor_range.slice_str(text.as_str()).to_owned();
                    ui.ctx().copy_text(text);
                    None
                }
            }
            Event::Cut => {
                if cursor_range.is_empty() {
                    None
                } else {
                    let copy_text = cursor_range.slice_str(text.as_str()).to_owned();
                    ui.ctx().copy_text(copy_text);
                    Some(CCursorRange::one(text.delete_selected(&cursor_range)))
                }
            }
            Event::Paste(text_to_insert) => {
                if !text_to_insert.is_empty() {
                    let mut ccursor = text.delete_selected(&cursor_range);

                    text.insert_text_at(&mut ccursor, text_to_insert, usize::MAX);

                    Some(CCursorRange::one(ccursor))
                } else {
                    None
                }
            }
            Event::Text(text_to_insert) => {
                // Newlines are handled by `Key::Enter`.
                if !text_to_insert.is_empty() && text_to_insert != "\n" && text_to_insert != "\r" {
                    let mut ccursor = text.delete_selected(&cursor_range);

                    text.insert_text_at(&mut ccursor, text_to_insert, usize::MAX);

                    autocompletion.requested = true;

                    Some(CCursorRange::one(ccursor))
                } else {
                    None
                }
            }
            Event::Key {
                key: Key::Space,
                pressed: true,
                modifiers,
                ..
            } if modifiers.command => {
                autocompletion.requested = true;
                None
            }
            Event::Key {
                key: Key::Tab,
                pressed: true,
                modifiers,
                ..
            } => {
                let mut ccursor = text.delete_selected(&cursor_range);
                if modifiers.shift {
                    // TODO(emilk): support removing indentation over a selection?
                    text.decrease_indentation(&mut ccursor);
                } else {
                    text.insert_text_at(&mut ccursor, "\t", usize::MAX);
                }
                Some(CCursorRange::one(ccursor))
            }
            Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } if return_key.is_some_and(|return_key| {
                *key == return_key.logical_key && modifiers.matches_logically(return_key.modifiers)
            }) =>
            {
                let mut ccursor = text.delete_selected(&cursor_range);
                text.insert_text_at(&mut ccursor, "\n", usize::MAX);
                // TODO(emilk): if code editor, auto-indent by same leading tabs, + one if the lines end on an opening bracket
                Some(CCursorRange::one(ccursor))
            }

            Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } if (modifiers.matches_logically(Modifiers::COMMAND) && *key == Key::Y)
                || (modifiers.matches_logically(Modifiers::SHIFT | Modifiers::COMMAND)
                    && *key == Key::Z) =>
            {
                if let Some((redo_ccursor_range, redo_txt)) = state
                    .undoer
                    .lock()
                    .redo(&(cursor_range, text.as_str().to_owned()))
                {
                    text.replace_with(redo_txt);
                    Some(*redo_ccursor_range)
                } else {
                    None
                }
            }

            Event::Key {
                key: Key::Z,
                pressed: true,
                modifiers,
                ..
            } if modifiers.matches_logically(Modifiers::COMMAND) => {
                if let Some((undo_ccursor_range, undo_txt)) = state
                    .undoer
                    .lock()
                    .undo(&(cursor_range, text.as_str().to_owned()))
                {
                    text.replace_with(undo_txt);
                    Some(*undo_ccursor_range)
                } else {
                    None
                }
            }

            Event::Key {
                modifiers,
                key,
                pressed: true,
                ..
            } => check_for_mutating_key_press(os, &cursor_range, text, galley, modifiers, *key),

            Event::Ime(ime_event) => match ime_event {
                ImeEvent::Enabled => {
                    state.ime_enabled = true;
                    state.ime_cursor_range = cursor_range;
                    None
                }
                ImeEvent::Preedit(text_mark) => {
                    if text_mark == "\n" || text_mark == "\r" {
                        None
                    } else {
                        // Empty prediction can be produced when user press backspace
                        // or escape during IME, so we clear current text.
                        let mut ccursor = text.delete_selected(&cursor_range);
                        let start_cursor = ccursor;
                        if !text_mark.is_empty() {
                            text.insert_text_at(&mut ccursor, text_mark, usize::MAX);
                        }
                        state.ime_cursor_range = cursor_range;
                        Some(CCursorRange::two(start_cursor, ccursor))
                    }
                }
                ImeEvent::Commit(prediction) => {
                    if prediction == "\n" || prediction == "\r" {
                        None
                    } else {
                        state.ime_enabled = false;

                        if !prediction.is_empty()
                            && cursor_range.secondary.index
                                == state.ime_cursor_range.secondary.index
                        {
                            let mut ccursor = text.delete_selected(&cursor_range);
                            text.insert_text_at(&mut ccursor, prediction, usize::MAX);
                            Some(CCursorRange::one(ccursor))
                        } else {
                            let ccursor = cursor_range.primary;
                            Some(CCursorRange::one(ccursor))
                        }
                    }
                }
                ImeEvent::Disabled => {
                    state.ime_enabled = false;
                    None
                }
            },

            _ => None,
        };

        if let Some(new_ccursor_range) = did_mutate_text {
            any_change = true;

            // Layout again to avoid frame delay, and to keep `text` and `galley` in sync.
            *galley = layouter(ui, text, wrap_width);

            // Set cursor_range using new galley:
            cursor_range = new_ccursor_range;
        }
    }

    state.cursor.set_char_range(Some(cursor_range));

    state.undoer.lock().feed_state(
        ui.input(|i| i.time),
        &(cursor_range, text.as_str().to_owned()),
    );

    (any_change, cursor_range)
}

// ----------------------------------------------------------------------------

fn remove_ime_incompatible_events(events: &mut Vec<Event>) {
    // Remove key events which cause problems while 'IME' is being used.
    // See https://github.com/emilk/egui/pull/4509
    events.retain(|event| {
        !matches!(
            event,
            Event::Key { repeat: true, .. }
                | Event::Key {
                    key: Key::Backspace
                        | Key::ArrowUp
                        | Key::ArrowDown
                        | Key::ArrowLeft
                        | Key::ArrowRight,
                    ..
                }
        )
    });
}

// ----------------------------------------------------------------------------

/// Returns `Some(new_cursor)` if we did mutate `text`.
fn check_for_mutating_key_press(
    os: OperatingSystem,
    cursor_range: &CCursorRange,
    text: &mut dyn TextBuffer,
    galley: &Galley,
    modifiers: &Modifiers,
    key: Key,
) -> Option<CCursorRange> {
    match key {
        Key::Backspace => {
            let ccursor = if modifiers.mac_cmd {
                text.delete_paragraph_before_cursor(galley, cursor_range)
            } else if let Some(cursor) = cursor_range.single() {
                if modifiers.alt || modifiers.ctrl {
                    // alt on mac, ctrl on windows
                    text.delete_previous_word(cursor)
                } else {
                    text.delete_previous_char(cursor)
                }
            } else {
                text.delete_selected(cursor_range)
            };
            Some(CCursorRange::one(ccursor))
        }

        Key::Delete if !modifiers.shift || os != OperatingSystem::Windows => {
            let ccursor = if modifiers.mac_cmd {
                text.delete_paragraph_after_cursor(galley, cursor_range)
            } else if let Some(cursor) = cursor_range.single() {
                if modifiers.alt || modifiers.ctrl {
                    // alt on mac, ctrl on windows
                    text.delete_next_word(cursor)
                } else {
                    text.delete_next_char(cursor)
                }
            } else {
                text.delete_selected(cursor_range)
            };
            let ccursor = CCursor {
                prefer_next_row: true,
                ..ccursor
            };
            Some(CCursorRange::one(ccursor))
        }

        Key::H if modifiers.ctrl => {
            let ccursor = text.delete_previous_char(cursor_range.primary);
            Some(CCursorRange::one(ccursor))
        }

        Key::K if modifiers.ctrl => {
            let ccursor = text.delete_paragraph_after_cursor(galley, cursor_range);
            Some(CCursorRange::one(ccursor))
        }

        Key::U if modifiers.ctrl => {
            let ccursor = text.delete_paragraph_before_cursor(galley, cursor_range);
            Some(CCursorRange::one(ccursor))
        }

        Key::W if modifiers.ctrl => {
            let ccursor = if let Some(cursor) = cursor_range.single() {
                text.delete_previous_word(cursor)
            } else {
                text.delete_selected(cursor_range)
            };
            Some(CCursorRange::one(ccursor))
        }

        _ => None,
    }
}
