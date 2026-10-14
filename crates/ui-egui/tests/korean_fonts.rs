use egui::vec2;
use egui_kittest::Harness;
use photocraft_ui_egui::PhotocraftApp;
use photocraft_ui_egui::theme::{self, ThemeKind, SYSTEM_KOREAN_FALLBACK};
use serde_json::json;

const KOREAN_TEXT: &str = "한국어 파일과 폴더 이름";

#[test]
fn system_korean_fallback_is_installed_in_all_families() {
    let ctx = egui::Context::default();
    ctx.run_ui(Default::default(), |ui| {
        theme::install_fonts(ui.ctx());
    })
    .textures_delta
    .clear();

    // In the second frame, the installed fonts are active.
    ctx.run_ui(Default::default(), |ui| {
        let off = std::env::var_os("PHOTOCRAFT_SYSTEM_FONTS").is_some_and(|v| v == "0");
        let has_korean = ui.ctx().fonts(|f| f.definitions().font_data.contains_key(SYSTEM_KOREAN_FALLBACK));
        if !has_korean && (off || !cfg!(target_os = "macos")) {
            return;
        }

        assert!(has_korean, "SYSTEM_KOREAN_FALLBACK should be in font_data");

        ui.ctx().fonts(|f| {
            let defs = f.definitions();
            for (family, stack) in &defs.families {
                assert!(
                    stack.iter().any(|name| name == SYSTEM_KOREAN_FALLBACK),
                    "{family:?} stack lacks {SYSTEM_KOREAN_FALLBACK}: {stack:?}"
                );
            }
        });

        let font_ids = [
            egui::FontId::proportional(13.0),
            egui::FontId::monospace(13.0),
            theme::medium(13.0),
            theme::semibold(17.0),
        ];

        ui.ctx().fonts_mut(|f| {
            for id in &font_ids {
                assert!(f.has_glyphs(id, KOREAN_TEXT), "{id:?} lacks glyphs for {KOREAN_TEXT}");
                let galley = f.layout_no_wrap(KOREAN_TEXT.to_string(), id.clone(), egui::Color32::WHITE);
                assert!(galley.size().x > 0.0);
            }
        });
    })
    .textures_delta
    .clear();
}

#[test]
fn korean_tab_title_and_layer_name_renders_without_tofu() {
    let mut h = Harness::builder().with_size(vec2(1000.0, 700.0)).with_max_steps(64).build_eframe(move |cc| {
        PhotocraftApp::setup_context(&cc.egui_ctx, ThemeKind::Pro);
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), photocraft_ui_egui::Services::default());
        app.run("file.new", json!({"width": 400, "height": 300, "name": "한국어_테스트_문서.psd"})).unwrap();
        app
    });
    h.run_steps(10);
    let ctx = h.ctx.clone();
    let has = ctx.fonts_mut(|f| f.has_glyphs(&egui::FontId::proportional(12.0), "한국어_테스트_문서.psd"));
    assert!(has, "NFC Korean glyphs should be loaded for tab title");

    // NFD decomposed Korean: "한" -> \u{1112}\u{1161}\u{11ab}
    let nfd_korean = "\u{1112}\u{1161}\u{11ab}\u{1100}\u{116e}\u{11a8}\u{110b}\u{1165}";
    let composed = photocraft_text::hangul::compose_hangul(nfd_korean);
    assert_eq!(composed, "한국어");

    let display_composed = photocraft_ui_egui::file_open::display_name(&format!("/Volumes/Data/{nfd_korean}/test.psd"));
    assert_eq!(display_composed, "test.psd");

    let folder_composed = photocraft_ui_egui::file_open::display_name(nfd_korean);
    assert_eq!(folder_composed, "한국어");
}
