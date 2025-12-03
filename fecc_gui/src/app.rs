// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
use std::cmp::PartialEq;
mod canvas_interaction;
mod eframe_ui;

use fecc_core::asset::{Asset, AssetType};
use fecc_core::character::Colourable::{
    Accessory, Cloth, EyeAndBeard, Hair, Leather, Metal, Skin, Trim,
};
use fecc_core::character::{Character, CharacterPart, ColourPalette, Colourable};
use fecc_core::export::ExportSize;
use fecc_core::file_io::{load_asset_libraries, load_colours_from_csv, load_image_bytes};
use fecc_core::types::Point;

use egui::ahash::{HashMap, HashSet};
use egui::{Align, Color32, ColorImage, Context, Pos2, Rect, Shape, Ui, Vec2, pos2, vec2};
use egui_notify::{Anchor, Toasts};
use futures_channel::mpsc;
use futures_util::future::join_all;
use image::RgbaImage;
use indexmap::IndexMap;
use std::path::PathBuf;
use std::sync::Arc;
use strum::IntoEnumIterator as _;
use strum_macros::EnumIter;

#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::Runtime;

type ImageReceiver = Option<mpsc::UnboundedReceiver<(String, Result<Arc<RgbaImage>, String>)>>;
pub(crate) type ImageSender = mpsc::UnboundedSender<(String, Result<Arc<RgbaImage>, String>)>;

#[derive(Debug, PartialEq, Clone, Copy, EnumIter, Eq, Hash)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

#[derive(PartialEq)]
enum CanvasType {
    Portrait,
    Token,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Interaction {
    Move,
    Rotate {
        start_grab_vec: Vec2,
    },
    Scale {
        corner: Corner,
        start_grab_vec: Vec2,
    },
    Flip,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub struct FitResult {
    pub max_side: f32,
    pub orientation: Orientation,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct FECharacterCreator {
    character: Character,
    #[serde(skip)]
    asset_libraries: std::collections::HashMap<AssetType, IndexMap<String, Asset>>,
    #[serde(skip)]
    texture_cache: HashMap<String, egui::TextureHandle>,

    active_tab: AssetType,
    new_active_tab: bool,
    randomise_used: bool,
    randomise_colours_too: bool,

    #[serde(skip)]
    search_queries: HashMap<AssetType, String>,
    colour_picker_open_state: HashMap<Colourable, bool>,
    outline_picker_open_state: HashMap<AssetType, bool>,
    portrait_rect: Rect,
    token_rect: Rect,

    export_size_selection: ExportSize,

    #[serde(skip)]
    colour_palettes: std::collections::HashMap<Colourable, ColourPalette>,
    #[serde(skip)]
    palettes_receiver: Option<
        futures_channel::oneshot::Receiver<std::collections::HashMap<Colourable, ColourPalette>>,
    >,
    #[serde(skip)]
    asset_libraries_receiver: Option<
        futures_channel::oneshot::Receiver<
            std::collections::HashMap<AssetType, IndexMap<String, Asset>>,
        >,
    >,
    #[serde(skip)]
    image_receiver: ImageReceiver,
    #[serde(skip)]
    image_sender: ImageSender,
    #[serde(skip)]
    images_in_flight: HashSet<String>,

    #[serde(skip)]
    toasts: Toasts,

    #[cfg(not(target_arch = "wasm32"))]
    #[serde(skip)]
    tokio_runtime: Arc<Runtime>,

    #[serde(skip)]
    selected_part: Option<AssetType>,
    #[serde(skip)]
    pub interaction: Option<Interaction>,
    #[serde(skip)]
    pub content_bounds_cache: HashMap<PathBuf, Rect>,

    assets_panel_expanded: bool,
    colour_panel_expanded: bool,
    export_panel_expanded: bool,
    save_load_panel_expanded: bool,

    #[serde(skip)]
    character_needs_asset_refresh: bool,
    #[serde(skip)]
    is_character_normalised: bool,

    #[serde(skip)]
    loaded_character_receiver: Option<mpsc::UnboundedReceiver<Result<Character, String>>>,
    #[serde(skip)]
    loaded_character_sender: mpsc::UnboundedSender<Result<Character, String>>,

    #[cfg(target_arch = "wasm32")]
    asset_upload_panel_expanded: bool,

    #[cfg(target_arch = "wasm32")]
    add_art_window_open: bool,

    #[serde(skip)]
    #[cfg(target_arch = "wasm32")]
    add_art_error: Option<String>,

    #[serde(skip)]
    #[cfg(target_arch = "wasm32")]
    new_user_asset_receiver: Option<mpsc::UnboundedReceiver<Result<Asset, String>>>,

    #[serde(skip)]
    #[cfg(target_arch = "wasm32")]
    new_user_asset_sender: mpsc::UnboundedSender<Result<Asset, String>>,

    #[serde(skip)]
    about_window_open: bool,
}

impl Default for FECharacterCreator {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        let (tx, rx) = mpsc::unbounded();
        let (loaded_character_sender, loaded_character_receiver) = mpsc::unbounded();

        Self {
            character: Default::default(),
            asset_libraries: Default::default(),
            texture_cache: Default::default(),
            active_tab: AssetType::Token,
            new_active_tab: true,
            randomise_used: false,
            randomise_colours_too: false,
            search_queries: Default::default(),
            colour_picker_open_state: [
                (Hair, false),
                (EyeAndBeard, false),
                (Skin, false),
                (Metal, false),
                (Trim, false),
                (Cloth, false),
                (Leather, false),
                (Accessory, false),
            ]
            .into_iter()
            .collect(),
            outline_picker_open_state: [
                (AssetType::Armour, false),
                (AssetType::Face, false),
                (AssetType::Hair, false),
                (AssetType::Accessory, false),
                (AssetType::Token, false),
            ]
            .into_iter()
            .collect(),
            portrait_rect: Rect::NOTHING,
            token_rect: Rect::NOTHING,
            export_size_selection: ExportSize::Original,
            colour_palettes: Default::default(),
            palettes_receiver: None,
            asset_libraries_receiver: None,
            image_receiver: None,
            image_sender: mpsc::unbounded().0,
            images_in_flight: Default::default(),
            #[cfg(not(target_arch = "wasm32"))]
            tokio_runtime: Arc::new(Runtime::new().expect("Failed to create Tokio runtime")),
            selected_part: None,
            interaction: None,
            content_bounds_cache: HashMap::default(),
            assets_panel_expanded: true,
            colour_panel_expanded: true,
            export_panel_expanded: false,
            save_load_panel_expanded: false,
            character_needs_asset_refresh: false,
            is_character_normalised: false,
            loaded_character_receiver: Some(loaded_character_receiver),
            loaded_character_sender,
            #[cfg(target_arch = "wasm32")]
            asset_upload_panel_expanded: false,

            #[cfg(target_arch = "wasm32")]
            add_art_window_open: false,

            #[cfg(target_arch = "wasm32")]
            add_art_error: None,

            #[cfg(target_arch = "wasm32")]
            new_user_asset_receiver: Some(rx),

            #[cfg(target_arch = "wasm32")]
            new_user_asset_sender: tx,

            toasts: Toasts::new().with_anchor(Anchor::BottomRight),
            about_window_open: false,
        }
    }
}

impl FECharacterCreator {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fe_character_creator: Self = if let Some(storage) = cc.storage {
            if let Some(mut saved_app) = eframe::get_value::<Self>(storage, eframe::APP_KEY) {
                saved_app.is_character_normalised = true;
                saved_app.character_needs_asset_refresh = true;
                saved_app
            } else {
                Default::default()
            }
        } else {
            Default::default()
        };

        #[cfg(not(target_arch = "wasm32"))]
        let tokio_runtime = fe_character_creator.tokio_runtime.clone();

        let (palettes_tx, palettes_rx) = futures_channel::oneshot::channel();
        let palettes_task = async move {
            let mut palettes = std::collections::HashMap::default();
            let mut futures = Vec::new();

            for colourable in
                Colourable::iter().filter(|&colourable| colourable != Colourable::Outline)
            {
                let filename = colourable.to_string() + "_colour_palette.csv";
                futures.push(async move { (colourable, load_colours_from_csv(&filename).await) });
            }

            let results = join_all(futures).await;

            for (colourable, result) in results {
                match result {
                    Ok(colours) => {
                        if !colours.is_empty() {
                            palettes.insert(colourable, ColourPalette::new(colours));
                        } else {
                            log::warn!("Loaded empty colour palette for {colourable}");
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to load colour palette for {colourable}: {e}");
                    }
                }
            }

            if palettes_tx.send(palettes).is_err() {
                log::warn!("Palettes receiver dropped before palettes were sent");
            }
        };

        let (assets_tx, assets_rx) = futures_channel::oneshot::channel();
        let assets_task = async move {
            match load_asset_libraries().await {
                Ok(libs) => {
                    if assets_tx.send(libs).is_err() {
                        log::warn!("Assets receiver dropped before libraries were sent");
                    }
                }
                Err(e) => {
                    log::error!("Failed to load asset libraries: {e}");
                }
            }
        };

        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(palettes_task);
            wasm_bindgen_futures::spawn_local(assets_task);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            tokio_runtime.spawn(palettes_task);
            tokio_runtime.spawn(assets_task);
        }

        let (image_sender, image_receiver) = mpsc::unbounded();
        let (loaded_character_sender, loaded_character_receiver) = mpsc::unbounded();

        fe_character_creator.palettes_receiver = Some(palettes_rx);
        fe_character_creator.asset_libraries_receiver = Some(assets_rx);
        fe_character_creator.image_receiver = Some(image_receiver);
        fe_character_creator.image_sender = image_sender;
        fe_character_creator.loaded_character_sender = loaded_character_sender;
        fe_character_creator.loaded_character_receiver = Some(loaded_character_receiver);

        fe_character_creator
    }

    fn get_or_load_texture(&mut self, ctx: &Context, asset: &Asset) -> Option<egui::TextureHandle> {
        if self.texture_cache.contains_key(&asset.id) {
            return self.texture_cache.get(&asset.id).cloned();
        }

        if asset.image_data.is_none() {
            if !self.images_in_flight.contains(&asset.id) {
                self.images_in_flight.insert(asset.id.clone());
                let sender = self.image_sender.clone();
                let path_buf = asset.path.clone();
                let ctx_clone = ctx.clone();
                let asset_id = asset.id.clone();

                let task = async move {
                    let result = match load_image_bytes(&path_buf).await {
                        Ok(bytes) => match image::load_from_memory(&bytes) {
                            Ok(img) => Ok(Arc::new(img.to_rgba8())),
                            Err(e) => Err(e.to_string()),
                        },
                        Err(e) => Err(e.to_string()),
                    };
                    sender
                        .unbounded_send((asset_id, result))
                        .expect("Failed to send image result");
                    ctx_clone.request_repaint();
                };

                #[cfg(target_arch = "wasm32")]
                wasm_bindgen_futures::spawn_local(task);

                #[cfg(not(target_arch = "wasm32"))]
                self.tokio_runtime.spawn(task);
            }

            return None;
        }

        if let Some(original_image_data) = &asset.image_data {
            let mut rgba_image = (**original_image_data).clone();
            fecc_core::recolour::recolour(
                &mut rgba_image,
                asset.asset_type,
                &self.character.character_colours,
                &self.character.outline_colours,
            );

            let size = [rgba_image.width() as usize, rgba_image.height() as usize];
            let pixels: Vec<Color32> = rgba_image
                .pixels()
                .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                .collect();
            let colour_image = ColorImage {
                size,
                pixels,
                ..Default::default()
            };

            let options = egui::TextureOptions {
                magnification: egui::TextureFilter::Nearest,
                minification: egui::TextureFilter::Nearest,
                wrap_mode: Default::default(),
                mipmap_mode: Default::default(),
            };
            let texture = ctx.load_texture(&asset.id, colour_image, options);
            self.texture_cache.insert(asset.id.clone(), texture.clone());
            Some(texture)
        } else {
            None
        }
    }

    fn display_assets(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        library: &IndexMap<String, Asset>,
        search_query: &str,
    ) -> Option<Asset> {
        let mut clicked_asset = None;

        let available_width = ui.available_width();
        let is_token = library
            .first()
            .map(|(_, a)| a.asset_type == AssetType::Token)
            .unwrap_or(false);
        let base_size = if is_token { 64.0 } else { 96.0 };

        let button_size_val = (available_width / base_size).floor().max(1.0) * base_size;
        let button_size = Vec2::splat(button_size_val);

        let spacing = ui.spacing().item_spacing.y;
        let label_height = 20.0;
        let total_item_size = vec2(button_size.x, button_size.y + spacing + label_height);

        for asset in library.iter().filter(|asset| {
            search_query.is_empty() || asset.1.name.to_lowercase().contains(search_query)
        }) {
            let (rect, response) = ui.allocate_at_least(total_item_size, egui::Sense::click());

            if ui.is_rect_visible(rect) {
                ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                    ui.vertical(|ui| {
                        let mut selected = false;
                        if let Some(part) = self.character.get_character_part(&asset.1.asset_type)
                            && part.asset == *asset.1
                        {
                            selected = true;
                        }

                        let main_texture_opt = self.get_or_load_texture(ctx, asset.1);

                        let button_response = ui.add(
                            egui::Button::new("")
                                .selected(selected)
                                .min_size(button_size),
                        );

                        if let Some(main_texture) = main_texture_opt {
                            let rect = button_response.rect;
                            let painter = ui.painter_at(rect);

                            if asset.1.asset_type == AssetType::Hair
                                && let Some(back_part_id) = &asset.1.back_part
                                && let Some(back_asset) = self.asset_libraries[&AssetType::HairBack]
                                    .get(back_part_id)
                                    .cloned()
                                && let Some(back_texture) =
                                    self.get_or_load_texture(ctx, &back_asset)
                            {
                                painter.image(
                                    back_texture.id(),
                                    rect,
                                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                    Color32::WHITE,
                                );
                            }

                            painter.image(
                                main_texture.id(),
                                rect,
                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        } else {
                            ui.painter_at(button_response.rect).text(
                                button_response.rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "Loading...",
                                egui::FontId::default(),
                                Color32::GRAY,
                            );
                        }

                        if selected && (self.randomise_used || self.new_active_tab) {
                            button_response.scroll_to_me(Some(Align::TOP));
                        }
                        if button_response.clicked() {
                            clicked_asset = Some(asset.1.clone());
                        }
                        ui.label(asset.1.name.clone());
                    });
                });
            } else {
                let mut selected = false;
                if let Some(part) = self.character.get_character_part(&asset.1.asset_type)
                    && part.asset == *asset.1
                {
                    selected = true;
                }

                if selected && (self.randomise_used || self.new_active_tab) {
                    response.scroll_to_me(Some(Align::TOP));
                }
            }
        }
        clicked_asset
    }

    fn select_asset(&mut self, asset: &Asset, asset_type: AssetType) {
        if self.is_asset_already_selected(asset) {
            self.deselect_asset(&asset.clone());
            return;
        }
        if asset_type == AssetType::Token {
            let rect = self.token_rect;
            let center = rect.center() - rect.min;
            self.character.set_character_part(
                &AssetType::Token,
                CharacterPart {
                    position: Point::new(center.x, center.y),
                    scale: (rect.height() / 64.0).floor().max(1.0),
                    rotation: 0.0,
                    flipped: false,
                    asset: asset.clone(),
                },
            );
        } else if asset_type == AssetType::Armour {
            let rect = self.portrait_rect;
            let scale = (rect.height() / 96.0).floor().max(1.0);
            let scaled_asset_height = 96.0 * scale;

            self.character.set_character_part(
                &AssetType::Armour,
                CharacterPart {
                    position: Point::new(
                        rect.width() / 2.0,
                        rect.height() - (scaled_asset_height / 2.0),
                    ),
                    scale,
                    rotation: 0.0,
                    flipped: false,
                    asset: asset.clone(),
                },
            );
        } else {
            let rect = self.portrait_rect;
            let center = rect.center() - rect.min;
            self.character.set_character_part(
                &asset_type,
                CharacterPart {
                    position: Point::new(center.x, center.y),
                    scale: (rect.height() / 96.0).floor().max(1.0),
                    rotation: 0.0,
                    flipped: false,
                    asset: asset.clone(),
                },
            );

            if asset_type == AssetType::Hair
                && let Some(back_part_id) = &asset.back_part
                && let Some(hair_part) = self.character.get_character_part(&AssetType::Hair)
                && let Some(back_asset) =
                    self.asset_libraries[&AssetType::HairBack].get(back_part_id)
            {
                self.character.set_character_part(
                    &AssetType::HairBack,
                    CharacterPart {
                        asset: back_asset.clone(),
                        flipped: hair_part.flipped,
                        ..hair_part.clone()
                    },
                );
            }
        }
    }

    fn deselect_asset(&mut self, asset: &Asset) {
        self.character.remove_character_part(&asset.asset_type);

        if asset.asset_type == AssetType::Hair {
            self.character.remove_character_part(&AssetType::HairBack);
        }
    }

    fn is_asset_already_selected(&self, asset: &Asset) -> bool {
        matches!(
            self.character.get_character_part(&asset.asset_type),
            Some(selected_asset) if asset == &selected_asset.asset
        )
    }

    fn scale_character_parts(&mut self, scale_factor: f32, is_token: bool) {
        let asset_types_to_scale = if is_token {
            vec![AssetType::Token]
        } else {
            vec![
                AssetType::Armour,
                AssetType::Face,
                AssetType::Hair,
                AssetType::HairBack,
                AssetType::Accessory,
            ]
        };

        for asset_type in asset_types_to_scale {
            if let Some(mut part) = self.character.get_character_part(&asset_type) {
                part.position.x *= scale_factor;
                part.position.y *= scale_factor;
                part.scale *= scale_factor;
                self.character.set_character_part(&asset_type, part);
            }
        }
    }

    fn get_normalised_character(&self) -> Character {
        let mut normalised_character = self.character.clone();

        let portrait_size = self.portrait_rect.size();
        if portrait_size.x > 0.0 && portrait_size.y > 0.0 {
            for asset_type in [
                AssetType::Armour,
                AssetType::Face,
                AssetType::Hair,
                AssetType::HairBack,
                AssetType::Accessory,
            ] {
                if let Some(mut part) = normalised_character.get_character_part(&asset_type) {
                    part.position.x /= portrait_size.x;
                    part.position.y /= portrait_size.y;
                    part.scale /= portrait_size.y;
                    normalised_character.set_character_part(&asset_type, part);
                }
            }
        }

        let token_size = self.token_rect.size();
        if token_size.x > 0.0
            && token_size.y > 0.0
            && let Some(mut part) = normalised_character.get_character_part(&AssetType::Token)
        {
            part.position.x /= token_size.x;
            part.position.y /= token_size.y;
            part.scale /= token_size.y;
            normalised_character.set_character_part(&AssetType::Token, part);
        }

        normalised_character
    }

    fn update_rect(&mut self, ctx: &Context, ui: &mut Ui) {
        let mut old_portrait_rect = self.portrait_rect;
        let mut old_token_rect = self.token_rect;

        let grid_spacing = vec2(
            ui.spacing().item_spacing.x * 1.5,
            ui.spacing().item_spacing.y * 2.0,
        );
        let available_size = ui.available_size_before_wrap();

        let fit_result = find_max_square_side(
            available_size.x,
            available_size.y,
            grid_spacing.x,
            grid_spacing.y,
        );

        let canvas_size = Vec2::splat(fit_result.max_side);

        egui::Grid::new("canvas_grid").show(ui, |ui| {
            let _portrait_rect = egui::Frame::canvas(ui.style())
                .inner_margin(0.0)
                .show(ui, |ui| {
                    self.paint_canvas(ctx, ui, CanvasType::Portrait, canvas_size)
                })
                .inner;

            if fit_result.orientation == Orientation::Vertical {
                ui.end_row();
            }

            let token_rect = egui::Frame::canvas(ui.style())
                .inner_margin(0.0)
                .show(ui, |ui| {
                    self.paint_canvas(ctx, ui, CanvasType::Token, canvas_size)
                })
                .inner;

            self.portrait_rect = _portrait_rect;
            self.token_rect = token_rect;
        });

        if self.is_character_normalised && self.portrait_rect.width() > 0.0 {
            let portrait_size = self.portrait_rect.size();
            if portrait_size.x > 0.0 && portrait_size.y > 0.0 {
                let portrait_parts = [
                    AssetType::Armour,
                    AssetType::Face,
                    AssetType::Hair,
                    AssetType::HairBack,
                    AssetType::Accessory,
                ];
                for asset_type in portrait_parts {
                    if let Some(mut part) = self.character.get_character_part(&asset_type) {
                        part.position.x *= portrait_size.x;
                        part.position.y *= portrait_size.y;
                        part.scale *= portrait_size.y;
                        self.character.set_character_part(&asset_type, part);
                    }
                }
            }

            let token_size = self.token_rect.size();
            if token_size.x > 0.0
                && token_size.y > 0.0
                && let Some(mut part) = self.character.get_character_part(&AssetType::Token)
            {
                part.position.x *= token_size.x;
                part.position.y *= token_size.y;
                part.scale *= token_size.y;
                self.character.set_character_part(&AssetType::Token, part);
            }
            self.is_character_normalised = false;
            old_portrait_rect = self.portrait_rect;
            old_token_rect = self.token_rect;
        }

        if old_portrait_rect.width() > 0.0
            && (old_portrait_rect.width() - self.portrait_rect.width()).abs() > 1.0
        {
            let scale_factor = self.portrait_rect.width() / old_portrait_rect.width();
            self.scale_character_parts(scale_factor, false);
        }
        if old_token_rect.width() > 0.0
            && (old_token_rect.width() - self.token_rect.width()).abs() > 1.0
        {
            let scale_factor = self.token_rect.width() / old_token_rect.width();
            self.scale_character_parts(scale_factor, true);
        }
    }

    fn paint_canvas(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        canvas_type: CanvasType,
        canvas_size: Vec2,
    ) -> Rect {
        let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
        let available_rect = response.rect;

        let side = available_rect.width().min(available_rect.height());
        let canvas_rect = Rect::from_center_size(available_rect.center(), Vec2::splat(side));

        painter.rect_filled(canvas_rect, 0.0, ui.style().visuals.extreme_bg_color);

        let parts_to_draw = if canvas_type == CanvasType::Token {
            vec![AssetType::Token]
        } else {
            vec![
                AssetType::HairBack,
                AssetType::Armour,
                AssetType::Face,
                AssetType::Hair,
                AssetType::Accessory,
            ]
        };

        self.handle_multi_touch(ctx);

        for &part_type in &parts_to_draw {
            if let Some(part) = self.character.get_character_part(&part_type)
                && let Some(texture) = self.get_or_load_texture(ctx, &part.asset)
            {
                let rect = Self::paint_transformed_part(&painter, &part, &texture, canvas_rect);

                if self.selected_part == Some(part_type) && part_type != AssetType::HairBack {
                    self.draw_interaction_handles(ui, rect, &part, response.rect, ctx);
                }
            }
        }

        if !parts_to_draw.contains(&AssetType::Token) {
            self.handle_interaction_beginning(&response, canvas_rect, &parts_to_draw);
            self.handle_ongoing_interactions(ctx, &response);
        }
        canvas_rect
    }

    fn paint_transformed_part(
        painter: &egui::Painter,
        part_data: &CharacterPart,
        texture: &egui::TextureHandle,
        canvas_rect: Rect,
    ) -> Rect {
        // Convert Point to Vec2 for egui
        let part_pos = vec2(part_data.position.x, part_data.position.y);
        let center_pos = (canvas_rect.min + part_pos).round();
        let scaled_size = texture.size_vec2() * part_data.scale;

        let mut mesh = egui::Mesh::with_texture(texture.id());
        let rect = Rect::from_center_size(center_pos, scaled_size);

        let uv = if part_data.flipped {
            Rect::from_min_max(pos2(1.0, 0.0), pos2(0.0, 1.0))
        } else {
            Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0))
        };

        mesh.add_rect_with_uv(rect, uv, Color32::WHITE);
        mesh.rotate(
            egui::emath::Rot2::from_angle(part_data.rotation),
            center_pos,
        );

        let transformed_rect = mesh.calc_bounds();
        painter.add(Shape::mesh(mesh));
        transformed_rect
    }

    fn is_pixel_opaque(part_data: &CharacterPart, check_pos: Pos2, canvas_rect: Rect) -> bool {
        if let Some(image_data) = &part_data.asset.image_data {
            let width = image_data.width() as f32;
            let height = image_data.height() as f32;
            let scaled_size = vec2(width, height) * part_data.scale;
            let part_pos = vec2(part_data.position.x, part_data.position.y);
            let center_pos = canvas_rect.min + part_pos;

            let p = check_pos - center_pos;
            let p = egui::emath::Rot2::from_angle(-part_data.rotation) * p;
            let p_top_left = p + scaled_size / 2.0;
            let image_coords = p_top_left / part_data.scale;

            let (x, y) = (image_coords.x.round() as u32, image_coords.y.round() as u32);

            if x < image_data.width() && y < image_data.height() {
                // Get pixel from RgbaImage
                let pixel = image_data.get_pixel(x, y);
                return pixel[3] > 0; // Check alpha
            }
        } else {
            log::warn!(
                "Pixel check failed: Image data not found in cache for path: {:?}",
                part_data.asset.id
            );
        }

        false
    }

    fn is_character_valid(&self, character: &Character) -> bool {
        let all_part_options = [
            character.get_character_part(&AssetType::Armour),
            character.get_character_part(&AssetType::Face),
            character.get_character_part(&AssetType::Hair),
            character.get_character_part(&AssetType::HairBack),
            character.get_character_part(&AssetType::Accessory),
            character.get_character_part(&AssetType::Token),
        ];

        for part in all_part_options.iter().flatten() {
            if self
                .asset_libraries
                .get(&part.asset.asset_type)
                .and_then(|lib| lib.get(&part.asset.id))
                .is_none()
            {
                log::error!(
                    "Validation failed: Asset '{}' of type '{:?}' not found in libraries.",
                    part.asset.id,
                    part.asset.asset_type
                );
                return false;
            }
        }
        true
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl FECharacterCreator {
    fn save_image(image: &image::RgbaImage, filename_stem: String) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG Image", &["png"])
            .set_file_name(&filename_stem)
            .save_file()
            && let Err(e) = image.save(path)
        {
            log::error!("Failed to save image: {e}");
        }
    }

    fn save_fecc(&self, filename_stem: String) {
        let normalised_character = self.get_normalised_character();
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("FECC Character", &["fecc"])
            .set_file_name(&filename_stem)
            .save_file()
        {
            match serde_json::to_string_pretty(&normalised_character) {
                Ok(json) => {
                    if let Err(e) = std::fs::write(path, json) {
                        log::error!("Failed to save FECC file: {e}");
                    }
                }
                Err(e) => {
                    log::error!("Failed to serialize character: {e}");
                }
            }
        }
    }

    fn load_fecc(&self) {
        let sender = self.loaded_character_sender.clone();
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("FECC Character", &["fecc"])
            .pick_file()
        {
            let result = std::fs::read_to_string(path)
                .map_err(|e| e.to_string())
                .and_then(|json| serde_json::from_str(&json).map_err(|e| e.to_string()));

            sender
                .unbounded_send(result)
                .expect("Failed to send loaded character");
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl FECharacterCreator {
    fn save_image(image: &image::RgbaImage, filename_stem: String) {
        use std::io::Cursor;

        let mut bytes: Vec<u8> = Vec::new();
        if let Err(e) = image.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png) {
            log::error!("Failed to encode image as PNG: {e}");
            return;
        }

        let filename = format!("{}.png", filename_stem);

        if let Err(e) = fecc_core::file_io::trigger_download(&bytes, &filename) {
            log::error!("Failed to trigger download: {e}");
        }
    }

    fn save_fecc(&self, filename_stem: String) {
        let normalised_character = self.get_normalised_character();
        match serde_json::to_string_pretty(&normalised_character) {
            Ok(json) => {
                let filename = format!("{}.fecc", filename_stem);
                if let Err(e) = fecc_core::file_io::trigger_download(json.as_bytes(), &filename) {
                    log::error!("Failed to trigger download: {e}");
                }
            }
            Err(e) => {
                log::error!("Failed to serialize character: {e}");
            }
        }
    }

    fn load_fecc(&self) {
        let sender = self.loaded_character_sender.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(file) = rfd::AsyncFileDialog::new()
                .add_filter("FECC Character", &["fecc"])
                .pick_file()
                .await
            {
                let bytes = file.read().await;
                let result = serde_json::from_slice(&bytes).map_err(|e| e.to_string());
                sender.unbounded_send(result).unwrap();
            }
        });
    }
}

fn find_max_square_side(x: f32, y: f32, padding_x: f32, padding_y: f32) -> FitResult {
    let s_h = ((x - padding_x) / 2.0).min(y);
    let s_v = x.min((y - padding_y) / 2.0);

    let s_h = s_h.max(0.0);
    let s_v = s_v.max(0.0);

    if s_h >= s_v {
        FitResult {
            max_side: s_h,
            orientation: Orientation::Horizontal,
        }
    } else {
        FitResult {
            max_side: s_v,
            orientation: Orientation::Vertical,
        }
    }
}
