use eframe::CreationContext;
use egui::{
    epaint::text::{FontInsert, InsertFontFamily},
    Vec2,
};
// MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].codepoints
// MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf
// MaterialSymbolsRounded[FILL,GRAD,opsz,wght].codepoints
// MaterialSymbolsRounded[FILL,GRAD,opsz,wght].ttf
// MaterialSymbolsSharp[FILL,GRAD,opsz,wght].codepoints
// MaterialSymbolsSharp[FILL,GRAD,opsz,wght].ttf

// let bytes = include_bytes!("../assets/fonts/Inter-VariableFont_opsz,wght.ttf");
fn add_icon_fonts(ctx: &egui::Context) {
    let mui_icon_fonts_sharp =
        include_bytes!("../assets/fonts/MaterialSymbolsSharp[FILL,GRAD,opsz,wght].ttf");

    ctx.add_font(FontInsert::new(
        "mui_sharp",
        egui::FontData::from_static(mui_icon_fonts_sharp),
        vec![InsertFontFamily {
            family: egui::FontFamily::Proportional,
            priority: egui::epaint::text::FontPriority::Lowest,
        }],
    ));
}

fn add_text_fonts(ctx: &egui::Context) {
    let bytes = include_bytes!("../assets/fonts/Inter-VariableFont_opsz,wght.ttf");
    ctx.add_font(FontInsert::new(
        "inter_body",
        egui::FontData::from_static(bytes),
        vec![InsertFontFamily {
            family: egui::FontFamily::Proportional,
            priority: egui::epaint::text::FontPriority::Highest,
        }],
    ));
}

fn apply_styles(style: &mut egui::Style) {
    // style.spacing.button_padding = Vec2::new(10.0, 6.0);
    style.spacing.combo_width = 8.0;
}

pub fn customize_app_styles(cc: &CreationContext<'_>) {
    let egui_ctx = &cc.egui_ctx;

    add_icon_fonts(egui_ctx);
    add_text_fonts(egui_ctx);

    for theme in [egui::Theme::Dark, egui::Theme::Light] {
        let mut style = std::sync::Arc::unwrap_or_clone(egui_ctx.style_of(theme));
        apply_styles(&mut style);
        egui_ctx.set_style_of(theme, style);
    }
}
