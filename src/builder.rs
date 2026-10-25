//! Window configuration, see [`WindowBuilder`].

use std::path::PathBuf;
use std::sync::Arc;

use crate::layer::{Layer, Anchor};
use crate::KeyboardInteractivity;
use crate::Error;

/// Builder for configuring and constructing a Wayland layer-shell window.
///
/// `WindowBuilder` follows the builder pattern, allowing ergonomic,
/// chained configuration of layer-shell parameters before the final
/// window object is created.
/// Use [`init_cpu`](WindowBuilder::init_cpu) for a window using cpu rendering
/// and [`init_wgpu`](WindowBuilder::init_wgpu) for a window using gpu rendering.
/// Both return a window implementing ratatui's
/// [`Backend`](ratatui_core::backend::Backend) trait.
///
/// This builder is designed for configuring:
/// - Size (width, height)
/// - Anchoring behavior (screen edges)
/// - Margins
/// - Exclusive zone (for panels/docks)
/// - Keyboard interactivity mode
/// - Layer (background, bottom, top, overlay)
/// - Namespace (for compositor identification)
/// - Font, font size and transparency
///
/// # Sizing
///
/// A width or height of `0` lets the compositor decide that dimension. This
/// is only allowed when the surface is anchored to both opposite edges
/// (left and right for the width, top and bottom for the height), otherwise
/// [`Error::InvalidConfiguration`] is returned.
///
/// # Example
///
/// A 30px high bar spanning the top of the screen:
///
/// ```no_run
/// use raclettui::{WindowBuilder, Anchor, Layer};
///
/// let window = WindowBuilder::new()
///     .set_namespace("my-panel")
///     .set_height(30)
///     .set_anchor(Anchor::Top)
///     .set_anchor(Anchor::Left)
///     .set_anchor(Anchor::Right)
///     .set_exclusive_zone(30)
///     .set_layer(Layer::Top)
///     .init_cpu()?;
/// # Ok::<(), raclettui::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct WindowBuilder {
    /// Namespace used by the compositor to identify the layer surface.
    /// This is typically used for grouping, debugging, or compositor rules.
    pub namespace: String,

    /// Desired width of the layer surface in logical pixels.
    /// A value of `0` lets the compositor decide, see [Sizing](#sizing).
    pub width: u32,

    /// Desired height of the layer surface in logical pixels.
    /// A value of `0` lets the compositor decide, see [Sizing](#sizing).
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
    /// If `None`, compositor defaults are used (no keyboard focus).
    pub keyboard_interactivity: Option<KeyboardInteractivity>,

    /// The layer in which this surface should be placed.
    /// Defaults to [`Layer::Top`].
    pub layer: Layer,

    /// Optional edge used for determining exclusive zone behavior.
    /// This should typically correspond to one of the set anchors.
    /// Requires layer shell version 5 and is ignored otherwise.
    pub exclusive_edge: Option<Anchor>,

    /// Optional path to a font file. When `None` the system monospace font is used.
    pub font_path: Option<PathBuf>,

    /// Font size in pixels, defaults to 17.
    pub font_size: f32,

    /// Opacity of cell backgrounds, from `0.0` (transparent) to `1.0` (opaque).
    pub bg_alpha: f32,

    /// Opacity of the text, from `0.0` (transparent) to `1.0` (opaque).
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
    /// - System monospace font at 17px
    /// - Fully opaque background and text
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

    /// Sets the desired width of the surface in logical pixels.
    pub fn set_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Sets the desired height of the surface in logical pixels.
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
        self.anchors.extend(anchors);
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
    /// This reserves screen space for the surface. A value of `-1` asks the
    /// compositor not to move the surface for other surfaces' exclusive zones.
    pub fn set_exclusive_zone(mut self, zone: i32) -> Self {
        self.exclusive_zone = Some(zone);
        self
    }

    /// Sets keyboard interactivity behavior.
    ///
    /// Keyboard events are only delivered with
    /// [`KeyboardInteractivity::Exclusive`] or [`KeyboardInteractivity::OnDemand`].
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

    /// Sets the path to a font file (`.ttf` / `.otf`).
    ///
    /// The font should be monospaced. When unset, the system monospace font is used.
    pub fn set_font_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.font_path = Some(path.into());
        self
    }

    /// Sets font size in pixels, default 17.0
    pub fn set_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Sets the opacity of cell backgrounds, clamped to `0.0..=1.0`.
    ///
    /// Values below `1.0` let the desktop show through the window.
    pub fn set_bg_alpha(mut self, alpha: f32) -> Self {
        self.bg_alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Sets the opacity of the text, clamped to `0.0..=1.0`.
    pub fn set_fg_alpha(mut self, alpha: f32) -> Self {
        self.fg_alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// All anchors combined into a single bitflag value.
    pub(crate) fn anchor_flags(&self) -> Anchor {
        self.anchors.iter().fold(Anchor::empty(), |acc, anchor| acc | *anchor)
    }

    /// Reads the configured font file, or the system monospace font.
    fn font_data(&self) -> Result<Vec<u8>, Error> {
        if let Some(path) = &self.font_path {
            return std::fs::read(path).map_err(Error::IoError);
        }

        let mut font_db = fontdb::Database::new();
        font_db.load_system_fonts();
        let query = fontdb::Query {
            families: &[fontdb::Family::Monospace],
            ..Default::default()
        };
        let font_id = font_db.query(&query).ok_or(Error::FontLoadingError)?;
        font_db
            .with_face_data(font_id, |data, _face_index| data.to_vec())
            .ok_or(Error::FontLoadingError)
    }

    /// Loads the font used by the cpu backend.
    ///
    /// Uses [`font_path`](Self::font_path) if set, otherwise the system monospace font.
    pub fn get_font(&self) -> Result<fontdue::Font, Error> {
        let data = self.font_data()?;
        fontdue::Font::from_bytes(data, fontdue::FontSettings::default())
            .map_err(|_| Error::FontLoadingError)
    }

    /// Creates the font system used by the wgpu backend.
    ///
    /// System fonts are always loaded so missing glyphs can fall back to them.
    /// If [`font_path`](Self::font_path) is set, that font is loaded too and
    /// used as the monospace family.
    pub fn get_font_system(&self) -> Result<glyphon::FontSystem, Error> {
        let mut font_system = glyphon::FontSystem::new();

        if let Some(path) = &self.font_path {
            let data = std::fs::read(path).map_err(Error::IoError)?;
            let db = font_system.db_mut();
            let ids = db.load_font_source(fontdb::Source::Binary(Arc::new(data)));
            let family = ids
                .first()
                .and_then(|id| db.face(*id))
                .and_then(|face| face.families.first())
                .map(|(name, _)| name.clone())
                .ok_or(Error::FontLoadingError)?;
            db.set_monospace_family(family);
        }

        Ok(font_system)
    }
}
