use eframe::egui::Vec2;

#[derive(Debug, Clone, Copy)]
pub(crate) struct LayoutScale {
    pub(crate) cell_size: f32,
    pub(crate) spacing: Vec2,
    pub(crate) padding: Vec2,
}

impl LayoutScale {
    pub(crate) const SPACING_FACTOR: Vec2 = Vec2::new(0.15, 0.20);
    pub(crate) const PADDING_FACTOR: Vec2 = Vec2::new(0.20, 0.30);

    #[must_use]
    pub(crate) fn new(cell_size: f32) -> Self {
        let spacing = Vec2::splat(cell_size) * Self::SPACING_FACTOR;
        let padding = Vec2::splat(cell_size) * Self::PADDING_FACTOR;
        Self {
            cell_size,
            spacing,
            padding,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ComponentUnits {
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl ComponentUnits {
    #[must_use]
    pub(crate) const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}
