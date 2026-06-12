use crate::layer::{Layer, Anchor};
use crate::KeyboardInteractivity;

#[derive(Debug)]
pub struct WindowBuilder {
    pub namespace: String,
    pub width: u32,
    pub height: u32,
    pub anchors: Vec<Anchor>,
    pub margin: Option<(i32, i32, i32, i32)>,
    pub exclusive_zone: Option<i32>,
    pub keyboard_interactivity: Option<KeyboardInteractivity>,
    pub layer: Layer,
    pub exclusive_edge: Option<Anchor>,
    pub font_path: Option<&'static str>,
    pub font_size: f32,
    pub bg_alpha: f32,
}

impl std::default::Default for WindowBuilder {
    fn default() -> Self {
        Self {
            namespace: "".to_string(),
            width: 0,
            height: 0,
            anchors: Vec::<Anchor>::new(),
            margin: None,
            exclusive_zone: None,
            keyboard_interactivity: None,
            layer: Layer::Top,
            exclusive_edge: None,
            font_path: None,
            font_size: 17.0,
            bg_alpha: 1.0,
        }
    }
}

impl WindowBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn set_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    pub fn set_anchor(mut self, anchor: Anchor) -> Self {
        self.anchors.push(anchor);
        self
    }

    pub fn set_anchors(mut self, anchors: Vec<Anchor>) -> Self {
        self.anchors.extend(anchors.iter());
        self
    }

    pub fn set_margin(mut self, top: i32, right: i32, bottom: i32, left: i32) -> Self {
        self.margin = Some((top, right, bottom, left));
        self
    }

    pub fn set_exclusive_zone(mut self, zone: i32) -> Self {
        self.exclusive_zone = Some(zone);
        self
    }

    pub fn set_keyboard_interactivity(
        mut self,
        keyboard_interactivity: KeyboardInteractivity,
    ) -> Self {
        self.keyboard_interactivity = Some(keyboard_interactivity);
        self
    }

    pub fn set_layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    pub fn set_exclusive_edge(mut self, edge: Anchor) -> Self {
        self.exclusive_edge = Some(edge);
        self
    }

    pub fn set_namespace(mut self, namespace: &str) -> Self {
        self.namespace = namespace.to_string();
        self
    }

    pub fn set_font_path(mut self, path: &'static str) -> Self {
        self.font_path = Some(path);
        self
    }

    pub fn set_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn set_bg_alpha(mut self, alpha: f32) -> Self {
        self.bg_alpha = alpha;
        self
    }
}
