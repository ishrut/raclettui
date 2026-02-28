use crate::layer::{Layer, Anchor};
use crate::KeyboardInteractivity;

/// Builder for configuring and constructing a Wayland layer-shell window.
///
/// `WindowBuilder` follows the builder pattern, allowing ergonomic,
/// chained configuration of layer-shell parameters before the final
/// window object is created.
/// Use `build_cpu()` for a window using cpu rendering and `build_wgpu()`
/// for a window using gpu rendering.
///
/// This builder is designed for configuring:
/// - Size (width, height)
/// - Anchoring behavior (screen edges)
/// - Margins
/// - Exclusive zone (for panels/docks)
/// - Keyboard interactivity mode
/// - Layer (background, bottom, top, overlay)
/// - Namespace (for compositor identification)
///
/// # Example
///
/// ```ignore
/// let window = WindowBuilder::new()
///     .set_namespace("my-panel")
///     .set_width(1920)
///     .set_height(30)
///     .set_anchor(Anchor::Top)
///     .set_anchor(Anchor::Left)
///     .set_anchor(Anchor::Right)
///     .set_exclusive_zone(30)
///     .set_layer(Layer::Top);
/// ```
#[derive(Debug)]
pub struct WindowBuilder {
    /// Namespace used by the compositor to identify the layer surface.
    /// This is typically used for grouping, debugging, or compositor rules.
    pub namespace: String,

    /// Desired width of the layer surface in logical pixels.
    /// A value of `0` typically indicates compositor-defined sizing.
    pub width: u32,

    /// Desired height of the layer surface in logical pixels.
    /// A value of `0` typically indicates compositor-defined sizing.
    pub height: u32,

    /// Anchors defining which edges of the screen the surface
    /// should be attached to.
    /// Multiple anchors can be combined (e.g., Top + Left + Right).
    pub anchors: Vec<Anchor>,

    /// Optional margins (top, right, bottom, left).
    /// When `None`, margins default to compositor-defined values (usually zero).
    pub margin: Option<(i32, i32, i32, i32)>,

    /// Optional exclusive zone in logical pixels.
    /// When set, this reserves screen space (commonly used by panels/docks).
    pub exclusive_zone: Option<i32>,

    /// Defines how the surface interacts with keyboard focus.
    /// If `None`, compositor defaults are used.
    pub keyboard_interactivity: Option<KeyboardInteractivity>,

    /// The layer in which this surface should be placed.
    /// Defaults to [`Layer::Top`].
    pub layer: Layer,

    /// Optional edge used for determining exclusive zone behavior.
    /// This should typically correspond to one of the set anchors.
    pub exclusive_edge: Option<Anchor>,

    /// Optional font path or will try to load from system
    pub font_path: Option<&'static str>,

    /// Font size defaults to 17.
    pub font_size: f32,

    /// Window transparency
    pub bg_alpha: f32,
    pub fg_alpha: f32,
}

impl std::default::Default for WindowBuilder {
    /// Creates a builder with sensible defaults:
    ///
    /// - Empty namespace
    /// - Width and height set to 0 (compositor decides)
    /// - No anchors
    /// - No margins
    /// - No exclusive zone
    /// - No keyboard interactivity override
    /// - Layer set to `Layer::Top`
    /// - No exclusive edge
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
            fg_alpha: 1.0
        }
    }
}

impl WindowBuilder {
    /// Creates a new `WindowBuilder` using default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the desired width of the surface.
    pub fn set_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Sets the desired height of the surface.
    pub fn set_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    /// Adds a single anchor edge to the surface.
    ///
    /// This method may be called multiple times to combine anchors.
    pub fn set_anchor(mut self, anchor: Anchor) -> Self {
        self.anchors.push(anchor);
        self
    }

    /// Adds multiple anchors to the surface.
    ///
    /// Existing anchors are preserved and the provided anchors are appended.
    pub fn set_anchors(mut self, anchors: Vec<Anchor>) -> Self {
        self.anchors.extend(anchors.iter());
        self
    }

    /// Sets surface margins in logical pixels.
    ///
    /// Order: `(top, right, bottom, left)`
    pub fn set_margin(mut self, top: i32, right: i32, bottom: i32, left: i32) -> Self {
        self.margin = Some((top, right, bottom, left));
        self
    }

    /// Sets the exclusive zone in logical pixels.
    ///
    /// This reserves screen space for the surface.
    pub fn set_exclusive_zone(mut self, zone: i32) -> Self {
        self.exclusive_zone = Some(zone);
        self
    }

    /// Sets keyboard interactivity behavior.
    pub fn set_keyboard_interactivity(
        mut self,
        keyboard_interactivity: KeyboardInteractivity,
    ) -> Self {
        self.keyboard_interactivity = Some(keyboard_interactivity);
        self
    }

    /// Sets the layer of the surface.
    pub fn set_layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    /// Sets the exclusive edge used for determining
    /// which anchor edge controls exclusive zone behavior.
    pub fn set_exclusive_edge(mut self, edge: Anchor) -> Self {
        self.exclusive_edge = Some(edge);
        self
    }

    /// Sets the namespace of the surface.
    ///
    /// This is typically used by compositors for identification or rules.
    pub fn set_namespace(mut self, namespace: &str) -> Self {
        self.namespace = namespace.to_string();
        self
    }

    /// Sets a path to a font
    pub fn set_font_path(mut self, path: &'static str) -> Self {
        self.font_path = Some(path);
        self
    }

    /// Sets font size, default 17.0
    pub fn set_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Sets window transparency
    pub fn bg_alpha(mut self, alpha: f32) -> Self {
        self.bg_alpha = alpha;
        self
    }

    pub fn fg_alpha(mut self, alpha: f32) -> Self {
        self.fg_alpha = alpha;
        self
    }

    // loads font from a path if provided or tries to default to system
    pub fn get_font(&self) -> fontdue::Font {
        if let Some(path) = self.font_path {
            let bytes = std::fs::read(path)
                .expect("Failed to read provided font file");
            fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).unwrap()

        } else {

            let mut font_db = fontdb::Database::new();
            font_db.load_system_fonts();

            let query = fontdb::Query {
                // Query for a monospace font
                families: &[fontdb::Family::Monospace],
                ..Default::default()
            };
            let font_id = font_db.query(&query).expect("src/cpu/buffer.rs No monospace font found"); // Find a font that matches the query
            let font_data = font_db
                .with_face_data(font_id, |data, _face_index| {
                    data.to_vec()
                })
                .expect("src/cpu/buffer.rs Failed to load font");

            fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default()).unwrap()
        }
    }
}
