use crate::extensions::color32::Contrast as _;
use crate::extensions::toggle_switch::toggle;
// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
use crate::FECharacterCreator;
use eframe::emath::vec2;
use eframe::epaint::{Color32, Stroke};
use egui::ahash::HashSet;
use egui::{Button, Context, Image, RichText, Ui};
use egui_extras::install_image_loaders;
use egui_extras::{Column, TableBuilder};
use fecc_core::asset::AssetType;
use fecc_core::character::Colourable::Skin;
use fecc_core::character::{CharacterPartColours, Colourable};
use fecc_core::export::{ExportSize, export_character};
use fecc_core::random::{randomize_assets, randomize_colours};
use fecc_core::types::Rgba;
use image::RgbaImage;
use strum::IntoEnumIterator as _;

// Helper functions for colour conversion
fn to_c32(c: Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

fn from_c32(c: Color32) -> Rgba {
    Rgba::new(c.r(), c.g(), c.b(), c.a())
}

impl eframe::App for FECharacterCreator {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        {
            let mut character = self.character.clone();
            let asset_libraries = &self.asset_libraries;
            let mut changed = false;

            for asset_type in AssetType::iter() {
                if let Some(part) = character.get_character_part(&asset_type)
                    && let Some(asset_from_lib) = asset_libraries
                        .get(&asset_type)
                        .and_then(|lib| lib.get(&part.asset.id))
                    && part.asset.image_data.is_none()
                    && asset_from_lib.image_data.is_some()
                {
                    let new_part = fecc_core::character::CharacterPart {
                        asset: asset_from_lib.clone(),
                        ..part
                    };
                    character.set_character_part(&asset_type, new_part);
                    changed = true;
                }
            }

            if changed {
                self.character = character;
            }
        }

        #[cfg(target_arch = "wasm32")]
        if let Some(mut rx) = self.new_user_asset_receiver.take() {
            if let Ok(Some(result)) = rx.try_next() {
                match result {
                    Ok(asset) => {
                        self.asset_libraries
                            .entry(asset.asset_type)
                            .or_default()
                            .insert(asset.id.clone(), asset);
                        self.add_art_error = None;
                    }
                    Err(e) => {
                        self.add_art_error = Some(e);
                    }
                }
            }
            self.new_user_asset_receiver = Some(rx);
        }

        if let Some(mut rx) = self.loaded_character_receiver.take() {
            if let Ok(Some(result)) = rx.try_next() {
                match result {
                    Ok(loaded_character) => {
                        if self.is_character_valid(&loaded_character) {
                            self.character = loaded_character;
                            self.is_character_normalised = true;
                            self.character_needs_asset_refresh = true;
                            self.texture_cache.clear();
                            self.toasts.success("Successfully loaded character.");
                        } else {
                            log::error!("Loaded character is invalid.");
                            self.toasts.error("Character is invalid.");
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to load character: {e}");
                        self.toasts.error("Failed to load character.");
                    }
                }
            }
            self.loaded_character_receiver = Some(rx);
        }

        egui::TopBottomPanel::top("top_toggle_bar")
            .resizable(false)
            .show(ctx, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    let asset_panel_icon = if self.assets_panel_expanded {
                        "◀"
                    } else {
                        "▶"
                    };
                    if ui
                        .button(asset_panel_icon)
                        .on_hover_text("Toggle Parts Panel")
                        .clicked()
                    {
                        self.assets_panel_expanded = !self.assets_panel_expanded;
                    }

                    egui::global_theme_preference_switch(ui);

                    if ui
                        .selectable_label(self.export_panel_expanded, "Export")
                        .clicked()
                    {
                        self.export_panel_expanded = !self.export_panel_expanded;
                        if self.export_panel_expanded {
                            self.save_load_panel_expanded = false;
                        }
                    }

                    if ui
                        .selectable_label(self.save_load_panel_expanded, "Save/Load")
                        .clicked()
                    {
                        self.save_load_panel_expanded = !self.save_load_panel_expanded;
                        if self.save_load_panel_expanded {
                            self.export_panel_expanded = false;
                        }
                    }

                    #[cfg(target_arch = "wasm32")]
                    if ui
                        .selectable_label(self.add_art_window_open, "Add Art")
                        .clicked()
                    {
                        self.add_art_window_open = !self.add_art_window_open;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let colour_panel_icon = if self.colour_panel_expanded {
                            "▶"
                        } else {
                            "◀"
                        };
                        if ui
                            .button(colour_panel_icon)
                            .on_hover_text("Toggle Colour Panel")
                            .clicked()
                        {
                            self.colour_panel_expanded = !self.colour_panel_expanded;
                        }

                        ui.separator();

                        if ui.button("About").clicked() {
                            self.about_window_open = !self.about_window_open;
                        }
                    });
                });
            });

        egui::SidePanel::left("part_selection_panel").show_animated(
            ctx,
            self.assets_panel_expanded,
            |ui| {
                ui.heading("Character Parts");
                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    for part_type in AssetType::get_selectable_part_types() {
                        if ui
                            .selectable_label(self.active_tab == part_type, part_type.to_string())
                            .clicked()
                        {
                            self.active_tab = part_type;
                            self.new_active_tab = true;
                        }
                    }
                    if ui.add(Button::new("Randomise")).clicked() {
                        self.randomise_used = true;

                        let types_to_randomize: Vec<AssetType> =
                            AssetType::get_selectable_part_types()
                                .filter(|asset_type| asset_type != &AssetType::Accessory)
                                .collect();

                        let canvas_size = fecc_core::types::Point::new(
                            self.portrait_rect.width(),
                            self.portrait_rect.height(),
                        );

                        randomize_assets(
                            &mut self.character,
                            &self.asset_libraries,
                            &types_to_randomize,
                            canvas_size,
                        );

                        if self.randomise_colours_too {
                            randomize_colours(&mut self.character, &self.colour_palettes);
                            self.texture_cache.clear();
                        }

                        self.character_needs_asset_refresh = true;
                    }

                    ui.add_space(5.0);
                    ui.add(toggle(&mut self.randomise_colours_too));
                    ui.label(if self.randomise_colours_too {
                        "Parts & Colours"
                    } else {
                        "Parts Only"
                    });
                });
                ui.separator();

                let search_query = self.search_queries.entry(self.active_tab).or_default();
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(search_query);
                });
                ui.separator();

                let search_query_cleaned = search_query.to_lowercase();

                if ui
                    .add(Button::new(
                        "Random ".to_owned() + &*self.active_tab.to_string(),
                    ))
                    .clicked()
                {
                    self.randomise_used = true;
                    let asset_type = self.active_tab;
                    let canvas_size = if asset_type == AssetType::Token {
                        fecc_core::types::Point::new(
                            self.token_rect.width(),
                            self.token_rect.height(),
                        )
                    } else {
                        fecc_core::types::Point::new(
                            self.portrait_rect.width(),
                            self.portrait_rect.height(),
                        )
                    };

                    randomize_assets(
                        &mut self.character,
                        &self.asset_libraries,
                        &[asset_type],
                        canvas_size,
                    );
                    self.character_needs_asset_refresh = true;
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let asset_type = self.active_tab;

                    if let Some(library) = self.asset_libraries.get(&asset_type)
                        && let Some(asset) =
                            self.display_assets(ctx, ui, &library.clone(), &search_query_cleaned)
                    {
                        self.select_asset(&asset.clone(), asset_type);
                    }
                });
            },
        );

        #[cfg(target_arch = "wasm32")]
        self.add_art_window(ctx);

        self.show_about_window(ctx);

        egui::SidePanel::right("colour_selection")
            .default_width(0.0)
            .show_animated(ctx, self.colour_panel_expanded, |ui| {
                self.update_stored_colour_palettes();
                self.update_stored_asset_libraries(ctx, ui);
                self.update_stored_image_data_cache();

                ui.add_space(5.0);

                if ui.button("Randomise Colours").clicked() {
                    randomize_colours(&mut self.character, &self.colour_palettes);
                    self.texture_cache.clear();
                }

                ui.add_space(5.0);
                let colour_picker_frame = egui::Frame {
                    inner_margin: egui::Margin::same(2),
                    outer_margin: egui::Margin::same(3),
                    shadow: Default::default(),
                    stroke: Stroke::new(1.0, Color32::GRAY),
                    ..Default::default()
                };

                for colourable in
                    Colourable::iter().filter(|&colourable| colourable != Colourable::Outline)
                {
                    colour_picker_frame.show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            let base_colour_c32 =
                                to_c32(self.character.character_colours[&colourable].base);
                            let text_colour = base_colour_c32
                                .find_contrasting_colour_on_background(
                                    ui.style().visuals.panel_fill,
                                );

                            let button_text = RichText::new(colourable.to_string())
                                .color(text_colour)
                                .size(13.0);

                            ui.horizontal(|ui| {
                                let button = Button::new(button_text)
                                    .fill(base_colour_c32)
                                    .stroke(Stroke::new(1.0, Color32::GRAY))
                                    .min_size(vec2(100.0, 20.0));

                                if ui.add(button).clicked() {
                                    *self
                                        .colour_picker_open_state
                                        .entry(colourable)
                                        .or_insert(false) ^= true;
                                }

                                if self.colour_palettes.contains_key(&colourable) {
                                    install_image_loaders(ctx);
                                    let cycle_colours_symbol = Image::new(egui::include_image!(
                                        "../../../assets/coins-swap.svg"
                                    ));
                                    let peek_colour = *self.colour_palettes[&colourable].peek();
                                    let cycle_colours = Button::image(cycle_colours_symbol)
                                        .fill(to_c32(peek_colour));

                                    if ui.add(cycle_colours).clicked() {
                                        self.character.character_colours.insert(
                                            colourable,
                                            CharacterPartColours::new(
                                                self.colour_palettes
                                                    .entry(colourable)
                                                    .or_default()
                                                    .next_cyclic(),
                                            ),
                                        );

                                        self.texture_cache.clear();
                                    }
                                }
                            });

                            if self.colour_picker_open_state[&colourable] {
                                self.present_colour_picker(ctx, &colourable);
                            }

                            egui::Grid::new(colourable).show(ui, |ui| {
                                let colour_part = self
                                    .character
                                    .character_colours
                                    .entry(colourable)
                                    .or_default();
                                let mut changed = false;

                                let mut lighter = to_c32(colour_part.lighter);
                                if ui.color_edit_button_srgba(&mut lighter).changed() {
                                    colour_part.lighter = from_c32(lighter);
                                    changed = true;
                                }

                                let mut neutral = to_c32(colour_part.neutral);
                                if ui.color_edit_button_srgba(&mut neutral).changed() {
                                    colour_part.neutral = from_c32(neutral);
                                    changed = true;
                                }

                                let mut darker = to_c32(colour_part.darker);
                                if ui.color_edit_button_srgba(&mut darker).changed() {
                                    colour_part.darker = from_c32(darker);
                                    changed = true;
                                }

                                if colourable == Skin {
                                    ui.end_row();
                                    let mut darker_darker = to_c32(colour_part.darker_darker);
                                    if ui.color_edit_button_srgba(&mut darker_darker).changed() {
                                        colour_part.darker_darker = from_c32(darker_darker);
                                        changed = true;
                                    }

                                    let mut darker_darker_darker =
                                        to_c32(colour_part.darker_darker_darker);
                                    if ui
                                        .color_edit_button_srgba(&mut darker_darker_darker)
                                        .changed()
                                    {
                                        colour_part.darker_darker_darker =
                                            from_c32(darker_darker_darker);
                                        changed = true;
                                    }
                                }
                                if changed {
                                    self.texture_cache.clear();
                                }
                            });
                        });
                    });
                }

                colour_picker_frame.show(ui, |ui| {
                    let outline_rgba = self
                        .character
                        .outline_colours
                        .get_outline_colour(self.active_tab);
                    let outline_c32 = to_c32(outline_rgba);
                    let text_colour = outline_c32
                        .find_contrasting_colour_on_background(ui.style().visuals.panel_fill);

                    let button_text = RichText::new(self.active_tab.to_string() + " Outline")
                        .color(text_colour)
                        .size(13.0);

                    let button = Button::new(button_text)
                        .fill(outline_c32)
                        .stroke(Stroke::new(1.0, Color32::GRAY))
                        .min_size(vec2(135.0, 20.0));

                    if ui.add(button).clicked() {
                        self.outline_picker_open_state.insert(self.active_tab, true);
                    }

                    egui::Window::new(self.active_tab.to_string() + " Outline Colour")
                        .open(
                            self.outline_picker_open_state
                                .get_mut(&self.active_tab)
                                .expect("Missing active_tab entry in outline_picker_open_state"),
                        )
                        .show(ctx, |ui| {
                            ui.label(
                                "Select a new ".to_owned()
                                    + &*self.active_tab.to_string()
                                    + " outline colour:",
                            );
                            ui.spacing_mut().slider_width = 275.0;

                            let mut current_outline_colour = to_c32(
                                self.character
                                    .outline_colours
                                    .get_outline_colour(self.active_tab),
                            );

                            let colour_changed = egui::widgets::color_picker::color_picker_color32(
                                ui,
                                &mut current_outline_colour,
                                egui::color_picker::Alpha::OnlyBlend,
                            );

                            if colour_changed {
                                self.character.outline_colours.set_outline_colour(
                                    self.active_tab,
                                    &from_c32(current_outline_colour),
                                );
                                self.texture_cache.clear();
                            }
                        });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Unique Colours: ");
                    let result = Self::analyse_combined_colours(
                        &export_character(
                            &self.character,
                            &[
                                AssetType::HairBack,
                                AssetType::Armour,
                                AssetType::Face,
                                AssetType::Hair,
                                AssetType::Accessory,
                            ],
                            (96, 96),
                            fecc_core::types::Point::new(
                                self.portrait_rect.width(),
                                self.portrait_rect.height(),
                            ),
                        ),
                        &export_character(
                            &self.character,
                            &[AssetType::Token],
                            (64, 64),
                            fecc_core::types::Point::new(
                                self.token_rect.width(),
                                self.token_rect.height(),
                            ),
                        ),
                    );

                    match result {
                        Ok((colour_count, has_semi_transparency)) => {
                            let count_colour = if colour_count > 15 {
                                Color32::RED
                            } else {
                                Color32::GREEN
                            };
                            ui.label(
                                RichText::new(colour_count.to_string())
                                    .color(count_colour)
                                    .strong(),
                            );

                            if has_semi_transparency {
                                ui.label(
                                    RichText::new(" (has semi-transparency!)")
                                        .color(Color32::RED)
                                        .strong(),
                                );
                            }
                        }
                        Err(_) => {
                            ui.label("Unknown");
                        }
                    }
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.label("Made with love and cats.");
                });
            });

        egui::TopBottomPanel::bottom("export").show_animated(
            ctx,
            self.export_panel_expanded,
            |ui| {
                ui.heading("Export");

                ui.horizontal(|ui| {
                    ui.label("Character Name:");
                    ui.text_edit_singleline(&mut self.character.name);
                });

                ui.horizontal(|ui| {
                    ui.label("Select Output Size:");
                    egui::ComboBox::from_label("")
                        .selected_text(self.export_size_selection.display_name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.export_size_selection,
                                ExportSize::Half,
                                ExportSize::Half.display_name(),
                            );
                            ui.selectable_value(
                                &mut self.export_size_selection,
                                ExportSize::Original,
                                ExportSize::Original.display_name(),
                            );
                            ui.selectable_value(
                                &mut self.export_size_selection,
                                ExportSize::Double,
                                ExportSize::Double.display_name(),
                            );
                        });
                });

                ui.separator();

                if ui
                    .button(format!(
                        "Export Portrait ({}x{})",
                        self.export_size_selection.portrait().0,
                        self.export_size_selection.portrait().1
                    ))
                    .clicked()
                    && let Some(image) = export_character(
                        &self.character,
                        &[
                            AssetType::HairBack,
                            AssetType::Armour,
                            AssetType::Face,
                            AssetType::Hair,
                            AssetType::Accessory,
                        ],
                        (
                            self.export_size_selection.portrait().0,
                            self.export_size_selection.portrait().1,
                        ),
                        fecc_core::types::Point::new(
                            self.portrait_rect.width(),
                            self.portrait_rect.height(),
                        ),
                    )
                {
                    Self::save_image(&image, self.character.name.clone() + "_portrait");
                }

                if ui
                    .button(format!(
                        "Export Token ({}x{})",
                        self.export_size_selection.token().0,
                        self.export_size_selection.token().1
                    ))
                    .clicked()
                    && let Some(image) = export_character(
                        &self.character,
                        &[AssetType::Token],
                        (
                            self.export_size_selection.token().0,
                            self.export_size_selection.token().1,
                        ),
                        fecc_core::types::Point::new(
                            self.token_rect.width(),
                            self.token_rect.height(),
                        ),
                    )
                {
                    Self::save_image(&image, self.character.name.clone() + "token");
                }
            },
        );

        egui::TopBottomPanel::bottom("save_load").show_animated(
            ctx,
            self.save_load_panel_expanded,
            |ui| {
                ui.heading("Save / Load");
                ui.horizontal(|ui| {
                    ui.label("Character Name:");
                    ui.text_edit_singleline(&mut self.character.name);
                });

                ui.separator();

                if ui.button("Save FECC").clicked() {
                    self.save_fecc(self.character.name.clone());
                }

                if ui.button("Load FECC").clicked() {
                    self.load_fecc();
                }
            },
        );

        egui::CentralPanel::default().show(ctx, |ui| {
            self.update_rect(ctx, ui);
        });

        self.new_active_tab = false;
        self.randomise_used = false;
        self.toasts.show(ctx);
        #[cfg(target_arch = "wasm32")]
        {
            self.add_art_error = None;
        }
    }

    /// Saves the application state to persistent storage.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let original_character = self.character.clone();
        self.character = self.get_normalised_character();
        eframe::set_value(storage, eframe::APP_KEY, self);
        self.character = original_character;
    }
}

impl FECharacterCreator {
    #[cfg(target_arch = "wasm32")]
    fn add_art_window(&mut self, ctx: &Context) {
        egui::Window::new("Add Art")
            .open(&mut self.add_art_window_open)
            .show(ctx, |ui| {
                ui.label("Upload a PNG file named in the format 'Name_Type.png'.");
                ui.label("For example: 'MyCoolFighter_Armour.png'");
                ui.add(
                    egui::Hyperlink::from_label_and_url(
                        "Guide on creating and adding your own art.",
                        "art",
                    )
                    .open_in_new_tab(true),
                );
                ui.separator();

                if ui.button("Upload File...").clicked() {
                    let sender = self.new_user_asset_sender.clone();
                    self.add_art_error = None; // Clear previous errors
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Some(file) = rfd::AsyncFileDialog::new().pick_file().await {
                            let file_name = file.file_name();
                            let bytes = file.read().await;
                            let result =
                                fecc_core::asset::Asset::try_from_bytes(&file_name, &*bytes);
                            sender.unbounded_send(result).unwrap();
                        }
                    });
                }

                if let Some(error) = &self.add_art_error {
                    log::error!("Failed to add art: {error}");
                    self.toasts.error("Failed to add art.");
                }
            });
    }

    fn show_about_window(&mut self, ctx: &Context) {
        egui::Window::new("About")
            .open(&mut self.about_window_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("FE Character Creator, 4th Edition");
                    ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                    ui.add_space(5.0);
                    ui.label("Copyright (C) 2025 aidan-es");
                    ui.add_space(8.0);

                    ui.label("This software comes with ABSOLUTELY NO WARRANTY.");
                    ui.label("Licensed under the GNU AGPLv3 - excluding art assets.");

                    ui.add_space(8.0);

                    ui.hyperlink_to("Source Code", "https://github.com/aidan-es/FECC4e");
                    ui.hyperlink_to("Full License", "https://www.gnu.org/licenses/agpl-3.0.html");
                    ui.add_space(10.0);
                    ui.label("Credits:");
                    ui.hyperlink_to("Rust", "https://www.rust-lang.org/");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.add_space(5.0);
                    ui.label(RichText::new(
                        "This is an update and full rewrite (in Rust) of the Fire Emblem Character Creator originally written in Java by TheFlyingMinotaur, updated by BaconMaster120 and converted to Scarla by ValeTheVioletMote."
                    ));
                    ui.add_space(5.0);
                    ui.label(RichText::new("Many art assets are by Iscaneus."));

                });
            });
    }

    fn update_stored_colour_palettes(&mut self) {
        if let Some(mut rx) = self.palettes_receiver.take() {
            match rx.try_recv() {
                Ok(Some(palettes)) => {
                    self.colour_palettes = palettes;
                    self.texture_cache.clear();
                }
                Ok(None) => {
                    self.palettes_receiver = Some(rx);
                }
                Err(_) => {
                    log::error!("Failed to receive palettes.");
                }
            }
        }
    }

    fn update_stored_asset_libraries(&mut self, ctx: &Context, _ui: &mut Ui) {
        if let Some(mut rx) = self.asset_libraries_receiver.take() {
            match rx.try_recv() {
                Ok(Some(libs)) => {
                    self.asset_libraries = libs;

                    if self.character_needs_asset_refresh {
                        for asset_type in AssetType::iter() {
                            if let Some(part) = self.character.get_character_part(&asset_type)
                                && let Some(asset_from_lib) = self
                                    .asset_libraries
                                    .get(&asset_type)
                                    .and_then(|lib| lib.get(&part.asset.id))
                            {
                                let new_part = fecc_core::character::CharacterPart {
                                    asset: asset_from_lib.clone(),
                                    ..part
                                };
                                self.character.set_character_part(&asset_type, new_part);
                            }
                        }
                        self.character_needs_asset_refresh = false;

                        // Also trigger image loading
                        let assets_to_load: Vec<_> = [
                            self.character.get_character_part(&AssetType::Armour),
                            self.character.get_character_part(&AssetType::Face),
                            self.character.get_character_part(&AssetType::Hair),
                            self.character.get_character_part(&AssetType::HairBack),
                            self.character.get_character_part(&AssetType::Accessory),
                            self.character.get_character_part(&AssetType::Token),
                        ]
                        .iter()
                        .flatten()
                        .map(|part| part.asset.clone())
                        .collect();

                        for asset in assets_to_load {
                            self.get_or_load_texture(ctx, &asset);
                        }
                    }

                    if let Some(hair_back_library) =
                        self.asset_libraries.get(&AssetType::HairBack).cloned()
                    {
                        for asset in hair_back_library.values() {
                            self.get_or_load_texture(ctx, asset);
                        }
                    }
                }
                Ok(None) => {
                    self.asset_libraries_receiver = Some(rx);
                }
                Err(_) => log::error!("Failed to receive asset libraries."),
            }
        }
    }

    fn update_stored_image_data_cache(&mut self) {
        if let Some(rx) = self.image_receiver.as_mut() {
            while let Ok(Some((id, result))) = rx.try_next() {
                self.images_in_flight.remove(&id);
                match result {
                    Ok(image_data) => {
                        let asset_type: Result<AssetType, _> =
                            id.rsplit_once('_').expect("REASON").1.parse();

                        match asset_type {
                            Ok(asset_type) => {
                                if let Some(asset) = self
                                    .asset_libraries
                                    .get_mut(&asset_type)
                                    .and_then(|lib| lib.get_mut(&id))
                                {
                                    asset.image_data = Some(image_data);
                                } else {
                                    log::warn!("Received image for unknown asset: {id}");
                                }
                            }
                            Err(e) => println!("Failed to parse: {e}"),
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to load image {id}: {e}");
                    }
                }
            }
        }
    }

    fn present_colour_picker(&mut self, ctx: &Context, colourable: &Colourable) {
        egui::Window::new(colourable.to_string() + " Colour")
            .open(
                self.colour_picker_open_state
                    .get_mut(colourable)
                    .expect("Missing colourable entry in colour_picker_open_state"),
            )
            .show(ctx, |ui| {
                ui.label("Select a new ".to_owned() + &*colourable.to_string() + " colour:");
                ui.spacing_mut().slider_width = 275.0;

                let colour_part = self
                    .character
                    .character_colours
                    .entry(*colourable)
                    .or_default();

                let mut base_c32 = to_c32(colour_part.base);
                let colour_changed = egui::widgets::color_picker::color_picker_color32(
                    ui,
                    &mut base_c32,
                    egui::color_picker::Alpha::OnlyBlend,
                );

                if colour_changed {
                    colour_part.set(from_c32(base_c32));
                    // derive_all_colours called inside set()
                    self.texture_cache.clear();
                }

                egui::CollapsingHeader::new("Colour Palette").show(ui, |ui| {
                    let columns = 9;
                    let palette_colours = self.colour_palettes[colourable].colours();
                    let rows = (palette_colours.len() as f32 / columns as f32).ceil() as usize;
                    let available_height = ui.available_height();
                    let table = TableBuilder::new(ui)
                        .striped(false)
                        .resizable(false)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .columns(Column::auto(), columns)
                        .min_scrolled_height(100.0)
                        .max_scroll_height(available_height);

                    table.body(|mut body| {
                        for i in 0..rows {
                            body.row(20.0, |mut row| {
                                for ii in 0..columns {
                                    row.col(|ui| {
                                        if i * columns + ii < palette_colours.len() {
                                            let colour = palette_colours[(i * columns) + ii];
                                            if ui
                                                .add(
                                                    Button::new("")
                                                        .min_size(vec2(20.0, 20.0))
                                                        .fill(to_c32(colour)),
                                                )
                                                .clicked()
                                            {
                                                self.character
                                                    .character_colours
                                                    .entry(*colourable)
                                                    .or_default()
                                                    .set(colour);
                                                self.texture_cache.clear();
                                            }
                                        }
                                    });
                                }
                            });
                        }
                    });
                });
            });
    }

    fn analyse_combined_colours(
        img_a: &Option<RgbaImage>,
        img_b: &Option<RgbaImage>,
    ) -> Result<(usize, bool), &'static str> {
        let img_a = img_a.as_ref().ok_or("Image A missing")?;
        let img_b = img_b.as_ref().ok_or("Image B missing")?;
        let mut unique_colours = HashSet::default();
        let mut has_semi_transparency = false;

        for pixel in img_a.pixels().chain(img_b.pixels()) {
            let alpha = pixel[3];
            if alpha != 0 {
                unique_colours.insert(pixel);
            }
            if !has_semi_transparency && (alpha > 0 && alpha < 255) {
                has_semi_transparency = true;
            }
        }

        Ok((unique_colours.len(), has_semi_transparency))
    }
}
