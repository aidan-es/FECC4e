use crate::app::{Corner, Interaction};
use crate::extensions::color32::Contrast as _;
// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
use crate::FECharacterCreator;
use eframe::emath::{Pos2, Rect, Rot2, Vec2, pos2, vec2};
use eframe::epaint::{Color32, Stroke};
use egui::Order::Background;
use egui::ahash::HashMap;
use egui::{Context, Id, LayerId, Painter, Response, Ui};
use fecc_core::asset::AssetType;
use fecc_core::character::CharacterPart;
use std::f32::consts::TAU;
use strum::IntoEnumIterator as _;

const HANDLE_RADIUS: f32 = 6.0;
const FLIP_HANDLE_RADIUS: f32 = 10.0;
const ROTATE_HANDLE_OFFSET: f32 = 20.0;

impl FECharacterCreator {
    fn get_content_bounds(&mut self, part: &CharacterPart, fallback_rect: Rect) -> Rect {
        if let Some(bounds) = self.content_bounds_cache.get(&part.asset.path) {
            return *bounds;
        }

        if let Some(image) = part.asset.image_data.as_ref() {
            let mut min_x = image.width();
            let mut max_x = 0;
            let mut min_y = image.height();
            let mut max_y = 0;
            let mut found_pixel = false;

            for (x, y, pixel) in image.enumerate_pixels() {
                if pixel[3] > 0 {
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                    min_y = min_y.min(y);
                    max_y = max_y.max(y);
                    found_pixel = true;
                }
            }

            if found_pixel {
                let centre_x = image.width() as f32 / 2.0;
                let centre_y = image.height() as f32 / 2.0;

                let min_pos = pos2(min_x as f32 - centre_x, min_y as f32 - centre_y);
                let max_pos = pos2((max_x + 1) as f32 - centre_x, (max_y + 1) as f32 - centre_y);

                let bounds = Rect::from_min_max(min_pos, max_pos);
                self.content_bounds_cache
                    .insert(part.asset.path.clone(), bounds);
                bounds
            } else {
                fallback_rect
            }
        } else {
            fallback_rect
        }
    }

    pub(crate) fn draw_interaction_handles(
        &mut self,
        ui: &Ui,
        part_rect: Rect,
        character_part: &CharacterPart,
        canvas_rect: Rect,
        ctx: &Context,
    ) {
        let mut content_bounds = self.get_content_bounds(character_part, part_rect);

        if character_part.flipped {
            content_bounds = Rect {
                min: pos2(-content_bounds.max.x, content_bounds.min.y),
                max: pos2(-content_bounds.min.x, content_bounds.max.y),
            };
        }

        let part_pos_vec = vec2(character_part.position.x, character_part.position.y);
        // Ensure geometric_centre_abs is Pos2
        let geometric_centre_abs = canvas_rect.min + part_pos_vec;
        let rot = Rot2::from_angle(character_part.rotation);

        let content_centre_offset_rel = content_bounds.center().to_vec2();
        let content_centre_abs =
            geometric_centre_abs + rot * (content_centre_offset_rel * character_part.scale);

        let corners_rel = [
            content_bounds.min.to_vec2(),
            vec2(content_bounds.max.x, content_bounds.min.y),
            content_bounds.max.to_vec2(),
            vec2(content_bounds.min.x, content_bounds.max.y),
        ];

        let corners_abs: [Pos2; 4] =
            corners_rel.map(|vec| geometric_centre_abs + rot * (vec * character_part.scale));

        let painter = Painter::new(
            ctx.clone(),
            LayerId::new(Background, Id::new(1)),
            canvas_rect,
        );

        let handle_stroke = Stroke::new(1.0, Color32::WHITE);
        let line_stroke = Stroke::new(1.0, Color32::from_gray(190));
        painter.line_segment([corners_abs[0], corners_abs[1]], line_stroke);
        painter.line_segment([corners_abs[1], corners_abs[2]], line_stroke);
        painter.line_segment([corners_abs[2], corners_abs[3]], line_stroke);
        painter.line_segment([corners_abs[3], corners_abs[0]], line_stroke);

        let mut scale_responses = HashMap::default();
        let mut rotate_responses = HashMap::default();
        let mut rotate_handle_positions = HashMap::default();
        let mut cursor_icon = egui::CursorIcon::Default;
        let mut wants_to_scale = false;
        let mut wants_to_rotate = false;

        let flip_handle_pos =
            corners_abs[1].lerp(corners_abs[2], 0.5) + rot * vec2(ROTATE_HANDLE_OFFSET, 0.0);
        let flip_rect =
            Rect::from_center_size(flip_handle_pos, vec2(1.0, 1.0) * FLIP_HANDLE_RADIUS * 2.0);
        let flip_response = ui.interact(flip_rect, ui.id().with("flip"), egui::Sense::click());

        let source = egui::Image::new(egui::include_image!("../../../assets/flip.svg"));
        let texture_id = source
            .load_for_size(ui.ctx(), Vec2::splat(24.0))
            .expect("Failed to load flip icon")
            .texture_id()
            .expect("Texture handle had no ID");

        let tint = ui
            .style()
            .visuals
            .extreme_bg_color
            .find_contrasting_colour();

        painter.image(
            texture_id,
            Rect::from_center_size(flip_handle_pos, Vec2::splat(24.0)),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            tint,
        );

        if flip_response.hovered() {
            cursor_icon = egui::CursorIcon::PointingHand;
        }

        for corner in Corner::iter() {
            let i = corner as usize;
            let corner_pos = corners_abs[i];

            let scale_rect =
                Rect::from_center_size(corner_pos, vec2(1.0, 1.0) * HANDLE_RADIUS * 2.0);
            let scale_response = ui.interact(
                scale_rect,
                ui.id().with("scale").with(i),
                egui::Sense::drag(),
            );

            painter.circle_filled(
                corner_pos,
                HANDLE_RADIUS,
                ui.style()
                    .visuals
                    .extreme_bg_color
                    .find_contrasting_colour(),
            );
            painter.circle_stroke(corner_pos, HANDLE_RADIUS, handle_stroke);

            if scale_response.hovered()
                || self
                    .interaction
                    .is_some_and(|i| matches!(i, Interaction::Scale{corner: c, ..} if c == corner))
            {
                let corner_vec = corner_pos.to_vec2() - content_centre_abs.to_vec2();
                let angle = corner_vec.angle();
                const THRESHOLD: f32 = std::f32::consts::PI / 8.0;

                if angle.abs() < THRESHOLD || (angle.abs() - std::f32::consts::PI).abs() < THRESHOLD
                {
                    cursor_icon = egui::CursorIcon::ResizeHorizontal;
                } else if (angle - std::f32::consts::FRAC_PI_2).abs() < THRESHOLD
                    || (angle + std::f32::consts::FRAC_PI_2).abs() < THRESHOLD
                {
                    cursor_icon = egui::CursorIcon::ResizeVertical;
                } else {
                    let corner_above_centre = corner_pos.y > content_centre_abs.y;
                    let corner_right_of_centre = corner_pos.x > content_centre_abs.x;

                    cursor_icon = match (corner_above_centre, corner_right_of_centre) {
                        (true, false) | (false, true) => egui::CursorIcon::ResizeNeSw,
                        (true, true) | (false, false) => egui::CursorIcon::ResizeNwSe,
                    };
                }
                wants_to_scale = true;
            }

            let diag_vec = vec2(
                if corner == Corner::TopLeft || corner == Corner::BottomLeft {
                    -1.0
                } else {
                    1.0
                },
                if corner == Corner::TopLeft || corner == Corner::TopRight {
                    -1.0
                } else {
                    1.0
                },
            );

            let rotated_diag = rot * diag_vec.normalized();
            let offset_vec = rotated_diag * ROTATE_HANDLE_OFFSET;
            let rotate_handle_pos = corner_pos + offset_vec;
            rotate_handle_positions.insert(corner, rotate_handle_pos);

            let rotate_rect =
                Rect::from_center_size(rotate_handle_pos, vec2(1.0, 1.0) * HANDLE_RADIUS * 2.0);
            let rotate_response = ui.interact(
                rotate_rect,
                ui.id().with("rotate").with(i),
                egui::Sense::drag(),
            );

            let rotation_angle = rotated_diag.rot90().angle();

            let source = egui::Image::new(egui::include_image!("../../../assets/rotate.svg"));
            let rotate_texture_id = source
                .load_for_size(ui.ctx(), Vec2::splat(20.0))
                .expect("Failed to load rotate icon")
                .texture_id()
                .expect("Texture handle had no ID");

            let centre = rotate_handle_pos;
            let half_size = 10.0;
            let rot = Rot2::from_angle(rotation_angle);
            let tint = ui
                .style()
                .visuals
                .extreme_bg_color
                .find_contrasting_colour();

            let vertices = [
                eframe::epaint::Vertex {
                    pos: centre + rot * vec2(-half_size, -half_size),
                    uv: pos2(0.0, 0.0),
                    color: tint,
                },
                eframe::epaint::Vertex {
                    pos: centre + rot * vec2(half_size, -half_size),
                    uv: pos2(1.0, 0.0),
                    color: tint,
                },
                eframe::epaint::Vertex {
                    pos: centre + rot * vec2(half_size, half_size),
                    uv: pos2(1.0, 1.0),
                    color: tint,
                },
                eframe::epaint::Vertex {
                    pos: centre + rot * vec2(-half_size, half_size),
                    uv: pos2(0.0, 1.0),
                    color: tint,
                },
            ];

            let indices = [0, 1, 2, 0, 2, 3];
            let mesh = eframe::epaint::Mesh {
                indices: indices.to_vec(),
                vertices: vertices.to_vec(),
                texture_id: rotate_texture_id,
            };
            painter.add(mesh);

            if rotate_response.hovered()
                || self
                    .interaction
                    .is_some_and(|i| matches!(i, Interaction::Rotate { .. }))
            {
                cursor_icon = if self
                    .interaction
                    .is_some_and(|i| matches!(i, Interaction::Rotate { .. }))
                {
                    egui::CursorIcon::Grabbing
                } else {
                    egui::CursorIcon::Grab
                };
                wants_to_rotate = true;
            }

            scale_responses.insert(corner, scale_response);
            rotate_responses.insert(corner, rotate_response);
        }

        if wants_to_scale || wants_to_rotate || flip_response.hovered() {
            ui.ctx().set_cursor_icon(cursor_icon);
        }

        if self.interaction.is_none() {
            let mut new_interaction = None;

            if flip_response.clicked() {
                new_interaction = Some(Interaction::Flip);
            }

            if new_interaction.is_none() {
                for corner in Corner::iter() {
                    if scale_responses[&corner].drag_started() {
                        let start_grab_vec = scale_responses[&corner]
                            .hover_pos()
                            .expect("Pointer outside of response area.")
                            .to_vec2();

                        new_interaction = Some(Interaction::Scale {
                            corner,
                            start_grab_vec,
                        });
                    }
                }
            }

            if new_interaction.is_none() {
                for corner in Corner::iter() {
                    if rotate_responses[&corner].drag_started() {
                        new_interaction = Some(Interaction::Rotate {
                            start_grab_vec: rotate_responses[&corner]
                                .hover_pos()
                                .expect("Pointer outside of response area.")
                                .to_vec2(),
                        });
                    }
                }
            }
            self.interaction = new_interaction;
        }
    }

    pub(crate) fn handle_interaction_beginning(
        &mut self,
        response: &Response,
        canvas_rect: Rect,
        parts_to_draw: &[AssetType],
    ) {
        if response.drag_started() {
            if self.interaction.is_none()
                && let Some(hover_pos) = response.hover_pos()
            {
                let mut part_found = None;
                for part_type in parts_to_draw.iter().rev() {
                    if let Some(part) = self.character.get_character_part(part_type)
                        && Self::is_pixel_opaque(&part, hover_pos, canvas_rect)
                    {
                        part_found = Some(*part_type);
                        break;
                    }
                }

                if let Some(part_type) = part_found {
                    let actual_part_type = if part_type == AssetType::HairBack {
                        AssetType::Hair
                    } else {
                        part_type
                    };
                    self.selected_part = Some(actual_part_type);
                    self.interaction = Some(Interaction::Move);
                } else {
                    self.selected_part = None;
                }
            }
        } else if response.clicked()
            && self.interaction.is_none()
            && let Some(hover_pos) = response.hover_pos()
        {
            let part_found = parts_to_draw.iter().rev().find(|&&part_type| {
                let part = self.character.get_character_part(&part_type);
                part.is_some_and(|p| Self::is_pixel_opaque(&p, hover_pos, canvas_rect))
            });

            self.selected_part = part_found.map(|&part_type| {
                if part_type == AssetType::HairBack {
                    AssetType::Hair
                } else {
                    part_type
                }
            });
        }
    }

    pub(crate) fn handle_ongoing_interactions(
        &mut self,
        ctx: &Context,
        canvas_response: &Response,
    ) {
        if self.interaction == Some(Interaction::Flip) {
            if let Some(selected_type) = self.selected_part
                && let Some(mut part) = self.character.get_character_part(&selected_type)
            {
                part.flipped = !part.flipped;

                let content_bounds = self.get_content_bounds(&part, Rect::ZERO);
                let content_centre_offset_rel = content_bounds.center().to_vec2();
                let rot = Rot2::from_angle(part.rotation);

                let x_offset = 2.0 * content_centre_offset_rel.x;
                let pos_correction = if part.flipped {
                    rot * (vec2(x_offset, 0.0) * part.scale)
                } else {
                    rot * (vec2(-x_offset, 0.0) * part.scale)
                };
                part.position.x += pos_correction.x;
                part.position.y += pos_correction.y;

                if selected_type == AssetType::Hair
                    && let Some(mut hair_back) =
                        self.character.get_character_part(&AssetType::HairBack)
                {
                    hair_back.flipped = !hair_back.flipped;
                    hair_back.position.x += pos_correction.x;
                    hair_back.position.y += pos_correction.y;
                    self.character
                        .set_character_part(&AssetType::HairBack, hair_back);
                }
                self.character.set_character_part(&selected_type, part);
            }
            self.interaction = None;
            return;
        }

        let Some(selected_type) = self.selected_part else {
            return;
        };

        let canvas_rect = canvas_response.rect;

        if let (Some(interaction), Some(current_pos)) =
            (&self.interaction, ctx.pointer_interact_pos())
        {
            let interaction_copy = *interaction;

            if let Some(mut part) = self.character.get_character_part(&selected_type)
                && ctx.input(|i| i.pointer.any_down())
            {
                let part_pos_vec = vec2(part.position.x, part.position.y);
                let geometric_centre_abs = canvas_rect.min + part_pos_vec;
                let old_rot = Rot2::from_angle(part.rotation);
                let old_scale = part.scale;

                let content_bounds = self.get_content_bounds(&part, Rect::ZERO);
                let content_centre_offset_rel = content_bounds.center().to_vec2();

                let centre =
                    geometric_centre_abs + old_rot * (content_centre_offset_rel * old_scale);

                match interaction_copy {
                    Interaction::Move => {
                        let delta = canvas_response.drag_delta();
                        if selected_type == AssetType::Hair
                            && let Some(mut hair_back) =
                                self.character.get_character_part(&AssetType::HairBack)
                        {
                            hair_back.position.x += delta.x;
                            hair_back.position.y += delta.y;
                            self.character
                                .set_character_part(&AssetType::HairBack, hair_back);
                        }
                        part.position.x += delta.x;
                        part.position.y += delta.y;
                    }
                    Interaction::Scale {
                        corner,
                        start_grab_vec,
                    } => {
                        let old_dist = start_grab_vec - centre.to_vec2();
                        let new_dist = current_pos.to_vec2() - centre.to_vec2();
                        if old_dist.length_sq() > 0.0 {
                            let scale_delta = new_dist.length() / old_dist.length();

                            part.scale *= scale_delta;
                            let pos_correction = old_rot
                                * (content_centre_offset_rel * old_scale * (1.0 - scale_delta));
                            part.position.x += pos_correction.x;
                            part.position.y += pos_correction.y;

                            if selected_type == AssetType::Hair
                                && let Some(mut hair_back) =
                                    self.character.get_character_part(&AssetType::HairBack)
                            {
                                hair_back.scale *= scale_delta;
                                hair_back.position.x += pos_correction.x;
                                hair_back.position.y += pos_correction.y;
                                self.character
                                    .set_character_part(&AssetType::HairBack, hair_back);
                            }
                        }
                        self.interaction = Some(Interaction::Scale {
                            corner,
                            start_grab_vec: current_pos.to_vec2(),
                        });
                    }

                    Interaction::Rotate { start_grab_vec } => {
                        let old_vec = start_grab_vec - centre.to_vec2();
                        let new_vec = current_pos - centre;
                        let angle_delta = new_vec.angle() - old_vec.angle();

                        part.rotation = (part.rotation + angle_delta).rem_euclid(TAU);
                        let new_rot = Rot2::from_angle(part.rotation);

                        let offset_vec = content_centre_offset_rel * part.scale;
                        let pos_correction = old_rot * offset_vec - new_rot * offset_vec;
                        part.position.x += pos_correction.x;
                        part.position.y += pos_correction.y;

                        if selected_type == AssetType::Hair
                            && let Some(mut hair_back) =
                                self.character.get_character_part(&AssetType::HairBack)
                        {
                            hair_back.rotation = part.rotation;
                            hair_back.position.x += pos_correction.x;
                            hair_back.position.y += pos_correction.y;
                            self.character
                                .set_character_part(&AssetType::HairBack, hair_back);
                        }

                        self.interaction = Some(Interaction::Rotate {
                            start_grab_vec: current_pos.to_vec2(),
                        });
                    }
                    Interaction::Flip => {
                        // This is now handled above
                    }
                }

                self.character.set_character_part(&selected_type, part);
            } else {
                self.interaction = None; // Drag released
            }
        }
    }

    pub(crate) fn handle_multi_touch(&mut self, ctx: &Context) {
        if let Some(multi_touch) = ctx.input(|i| i.multi_touch())
            && let Some(selected) = self.selected_part
            && let Some(mut part) = self.character.get_character_part(&selected)
        {
            let old_rot = Rot2::from_angle(part.rotation);
            let old_scale = part.scale;

            let content_bounds = self.get_content_bounds(&part, Rect::ZERO);
            let content_centre_offset_rel = content_bounds.center().to_vec2();

            let new_scale = old_scale * multi_touch.zoom_delta;
            let new_rot = old_rot * Rot2::from_angle(multi_touch.rotation_delta);
            part.scale = new_scale;
            part.rotation = new_rot.angle();

            let pos_correction = old_rot * (content_centre_offset_rel * old_scale)
                - new_rot * (content_centre_offset_rel * new_scale);
            part.position.x += pos_correction.x;
            part.position.y += pos_correction.y;

            self.character.set_character_part(&selected, part.clone());

            if selected == AssetType::Hair
                && let Some(mut hair_back) = self.character.get_character_part(&AssetType::HairBack)
            {
                hair_back.scale = new_scale;
                hair_back.rotation = new_rot.angle();
                hair_back.position.x += pos_correction.x;
                hair_back.position.y += pos_correction.y;
                self.character
                    .set_character_part(&AssetType::HairBack, hair_back);
            }
        }
    }
}
