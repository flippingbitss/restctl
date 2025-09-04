use crate::tiles::{Pane, PaneKind};

#[derive(PartialEq, Eq)]
pub enum RequestViewLayoutKind {
    Postman,
    Bruno,
    Custom(String),
}

pub struct RequestViewLayout {
    pub name: String,
    pub tree: egui_tiles::Tree<Pane>,
}

pub fn create_default() -> RequestViewLayout {
    let mut next_view_nr = 1;
    let mut gen_view = |kind: PaneKind| {
        let view = Pane::from_values(next_view_nr, kind);
        next_view_nr += 1;
        view
    };
    let mut tiles = egui_tiles::Tiles::default();
    let mut request_tabs = vec![];
    request_tabs.push({
        let auth = tiles.insert_pane(gen_view(PaneKind::Auth));
        let params = tiles.insert_pane(gen_view(PaneKind::QueryParams));
        let headers = tiles.insert_pane(gen_view(PaneKind::Headers));
        let body = tiles.insert_pane(gen_view(PaneKind::Body));
        tiles.insert_horizontal_tile(vec![auth, params, headers, body])
    });

    let mut response_tabs = vec![];
    response_tabs.push({
        let left = tiles.insert_pane(gen_view(PaneKind::ResponseStats));
        let middle = tiles.insert_pane(gen_view(PaneKind::ResponseHeaders));
        let right = tiles.insert_pane(gen_view(PaneKind::ResponseBody));

        tiles.insert_horizontal_tile(vec![left, middle, right])
    });

    let request_container = tiles.insert_tab_tile(request_tabs);
    let response_container = tiles.insert_tab_tile(response_tabs);
    let root = tiles.insert_vertical_tile(vec![request_container, response_container]);

    RequestViewLayout {
        name: "Default".to_string(),
        tree: egui_tiles::Tree::new("request_tree_default", root, tiles),
    }
}

pub fn create_postman() -> RequestViewLayout {
    let mut next_view_nr = 1;
    let mut gen_view = |kind: PaneKind| {
        let view = Pane::from_values(next_view_nr, kind);
        next_view_nr += 1;
        view
    };
    let mut tiles = egui_tiles::Tiles::default();
    //
    //    //   #12: Linear
    // #1: Tabs
    //   #14: Pane 2 - Query Params
    //   #13: Pane 1 - Auth
    //   #15: Pane 3 - Headers
    //   #21: Pane 4 - Body
    // #6: Tabs
    //   #17: Pane 5 - Response Stats
    //   #18: Pane 6 - Response Headers
    //   #19: Pane 7 - Response Body
    //
    let mut request_tabs = vec![];
    request_tabs.push({
        let params = tiles.insert_pane(gen_view(PaneKind::QueryParams));
        let auth = tiles.insert_pane(gen_view(PaneKind::Auth));
        let headers = tiles.insert_pane(gen_view(PaneKind::Headers));
        let body = tiles.insert_pane(gen_view(PaneKind::Body));
        tiles.insert_tab_tile(vec![params, auth, headers, body])
    });

    let mut response_tabs = vec![];
    response_tabs.push({
        let stats = tiles.insert_pane(gen_view(PaneKind::ResponseStats));
        let headers = tiles.insert_pane(gen_view(PaneKind::ResponseHeaders));
        let body = tiles.insert_pane(gen_view(PaneKind::ResponseBody));
        tiles.insert_tab_tile(vec![stats, headers, body])
    });

    let request_container = tiles.insert_tab_tile(request_tabs);
    let response_container = tiles.insert_tab_tile(response_tabs);
    let root = tiles.insert_vertical_tile(vec![request_container, response_container]);

    RequestViewLayout {
        name: "Postman".to_string(),
        tree: egui_tiles::Tree::new("request_tree_postman", root, tiles),
    }
}

pub fn create_bruno() -> RequestViewLayout {
    let mut next_view_nr = 1;
    let mut gen_view = |kind: PaneKind| {
        let view = Pane::from_values(next_view_nr, kind);
        next_view_nr += 1;
        view
    };
    let mut tiles = egui_tiles::Tiles::default();
    //
    //    //   #12: Linear
    // #1: Tabs
    //   #14: Pane 2 - Query Params
    //   #13: Pane 1 - Auth
    //   #15: Pane 3 - Headers
    //   #21: Pane 4 - Body
    // #6: Tabs
    //   #17: Pane 5 - Response Stats
    //   #18: Pane 6 - Response Headers
    //   #19: Pane 7 - Response Body
    //
    let mut request_tabs = vec![];
    request_tabs.push({
        let params = tiles.insert_pane(gen_view(PaneKind::QueryParams));
        let body = tiles.insert_pane(gen_view(PaneKind::Body));
        let headers = tiles.insert_pane(gen_view(PaneKind::Headers));
        let auth = tiles.insert_pane(gen_view(PaneKind::Auth));
        tiles.insert_tab_tile(vec![params, body, headers, auth])
    });

    let mut response_tabs = vec![];
    response_tabs.push({
        let stats = tiles.insert_pane(gen_view(PaneKind::ResponseStats));
        let headers = tiles.insert_pane(gen_view(PaneKind::ResponseHeaders));
        let body = tiles.insert_pane(gen_view(PaneKind::ResponseBody));
        tiles.insert_tab_tile(vec![body, headers, stats])
    });

    let request_container = tiles.insert_tab_tile(request_tabs);
    let response_container = tiles.insert_tab_tile(response_tabs);
    let root = tiles.insert_horizontal_tile(vec![request_container, response_container]);

    RequestViewLayout {
        name: "Bruno".to_string(),
        tree: egui_tiles::Tree::new("request_tree_bruno", root, tiles),
    }
}
