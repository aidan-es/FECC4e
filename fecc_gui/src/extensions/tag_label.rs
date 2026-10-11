// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
use egui::{
    Atom, Button, Color32, CornerRadius, Painter, Rect, Response, Stroke, TextStyle, Ui, Vec2,
    Visuals, WidgetText, pos2,
};

/// Represents the nature of the tag.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TagKind {
    Category, // Presently this is just the game, could be expanded.
    Artist,
}

/// Adds selectable labels customised for contributor tags.
pub(crate) trait TagLabel {
    fn tag_label(&mut self, kind: TagKind, selected: bool, text: impl Into<WidgetText>)
    -> Response;

    fn tag_plain_label(
        &mut self,
        kind: TagKind,
        selected: bool,
        text: impl Into<WidgetText>,
    ) -> Response;
}

impl TagLabel for Ui {
    fn tag_label(
        &mut self,
        kind: TagKind,
        selected: bool,
        text: impl Into<WidgetText>,
    ) -> Response {
        if kind != TagKind::Artist {
            return self.tag_plain_label(kind, selected, text);
        }
        with_tag_style(self, kind, |ui| {
            let icon_id = ui.next_auto_id().with("artist_icon");
            let text_style = ui
                .style()
                .override_text_style
                .clone()
                .unwrap_or(TextStyle::Button);
            let icon_size = Vec2::splat(ui.text_style_height(&text_style));
            let response =
                Button::selectable(selected, (Atom::custom(icon_id, icon_size), text.into()))
                    .atom_ui(ui);

            if let Some(rect) = response.rect(icon_id) {
                let colour = ui
                    .style()
                    .interact_selectable(&response, selected)
                    .text_color();
                paint_person(ui.painter(), rect, colour);
            }
            response.response
        })
    }

    fn tag_plain_label(
        &mut self,
        kind: TagKind,
        selected: bool,
        text: impl Into<WidgetText>,
    ) -> Response {
        with_tag_style(self, kind, |ui| ui.selectable_label(selected, text))
    }
}

/// Applies the style for `kind` while running `add`, then restores the previous style.
fn with_tag_style<R>(ui: &mut Ui, kind: TagKind, add: impl FnOnce(&mut Ui) -> R) -> R {
    let saved = ui.style().clone();
    tag_visuals(ui.visuals_mut(), kind);
    let result = add(ui);
    ui.set_style(saved);
    result
}

/// Sets the accent colour for `kind` in the light or dark theme. Artists also get square corners.
fn tag_visuals(visuals: &mut Visuals, kind: TagKind) {
    let accent = match kind {
        TagKind::Artist => visuals.warn_fg_color,
        TagKind::Category if visuals.dark_mode => Color32::from_rgb(190, 150, 255),
        TagKind::Category => Color32::from_rgb(120, 60, 190),
    };
    let ground = visuals.panel_fill;

    visuals.selection.bg_fill = ground.lerp_to_gamma(accent, 0.45);
    visuals.selection.stroke = Stroke::new(1.0, visuals.strong_text_color());

    let widgets = &mut visuals.widgets;
    for (state, fill) in [
        (&mut widgets.inactive, 0.0),
        (&mut widgets.hovered, 0.15),
        (&mut widgets.active, 0.3),
    ] {
        if kind == TagKind::Artist {
            state.corner_radius = CornerRadius::ZERO;
        }
        state.fg_stroke.color = accent;
        state.weak_bg_fill = ground.lerp_to_gamma(accent, fill);
        if fill > 0.0 {
            state.bg_stroke = Stroke::new(1.0, accent);
        }
    }
}

/// Paints a person icon (head and shoulders) in `rect`.
fn paint_person(painter: &Painter, rect: Rect, colour: Color32) {
    let size = rect.height();
    painter.circle_filled(
        pos2(rect.center().x, rect.top() + size * 0.3),
        size * 0.2,
        colour,
    );
    let shoulders = Rect::from_min_max(
        pos2(rect.left() + size * 0.15, rect.top() + size * 0.58),
        pos2(rect.right() - size * 0.15, rect.top() + size * 0.9),
    );
    let round = (shoulders.height() * 0.9) as u8;
    let corner_radius = CornerRadius {
        nw: round,
        ne: round,
        sw: 1,
        se: 1,
    };
    painter.rect_filled(shoulders, corner_radius, colour);
}
