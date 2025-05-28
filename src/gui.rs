use eframe::egui;
use lightning::offers::offer::{Amount, Offer, Quantity};
use std::str::FromStr;

pub struct Bolt12OfferDecoderApp {
    pub offer: Option<Offer>,
    input_text: String,
    error_message: Option<String>,
}

impl Default for Bolt12OfferDecoderApp {
    fn default() -> Self {
        let default_input_text = String::from(
            "lno1pqps7sjqpgt+yzm3qv4uxzmtsd3jjqer9wd3hy6tsw3+5k7msjzfpy7nz5yqcn+ygrfdej82um5wf5k2uckyypwa3eyt44h6txtxquqh7lz5djge4afgfjn7k4rgrkuag0jsd+5xvxg",
        );
        Self {
            offer: None,
            input_text: default_input_text,
            error_message: None,
        }
    }
}

impl eframe::App for Bolt12OfferDecoderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let lightning_blue = egui::Color32::from_rgb(30, 144, 255);
        let lightning_yellow = egui::Color32::from_rgb(255, 215, 0);
        let lightning_purple = egui::Color32::from_rgb(138, 43, 226);
        let dark_bg = egui::Color32::from_rgb(25, 25, 35);
        let card_bg = egui::Color32::from_rgb(35, 35, 50);
        let text_secondary = egui::Color32::from_rgb(160, 160, 180);

        let mut style = (*ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.panel_fill = dark_bg;
        style.visuals.extreme_bg_color = card_bg;
        style.visuals.widgets.noninteractive.bg_fill = card_bg;
        style.visuals.widgets.inactive.bg_fill = card_bg;
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 45, 65);
        style.visuals.widgets.active.bg_fill = lightning_blue;
        style.visuals.selection.bg_fill = lightning_blue;
        style.spacing.item_spacing = egui::vec2(12.0, 8.0);
        style.spacing.button_padding = egui::vec2(16.0, 8.0);
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);

            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                let title = egui::RichText::new("⚡ BOLT12 Offer Decoder ⚡")
                    .size(32.0)
                    .color(lightning_yellow)
                    .strong();
                ui.label(title);

                let subtitle = egui::RichText::new("Lightning Network Offer Analysis Tool")
                    .size(14.0)
                    .color(text_secondary)
                    .italics();
                ui.label(subtitle);
                ui.add_space(15.0);
            });

            egui::Frame::new()
                .fill(card_bg)
                .corner_radius(egui::CornerRadius::same(12))
                .inner_margin(egui::Margin::same(20))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 80)))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("🔍 Enter BOLT12 Offer:")
                                .size(16.0)
                                .color(lightning_blue)
                                .strong(),
                        );
                    });

                    ui.add_space(8.0);

                    let text_edit = egui::TextEdit::multiline(&mut self.input_text)
                        .desired_width(f32::INFINITY)
                        .desired_rows(4)
                        .background_color(egui::Color32::from_rgb(25, 25, 40))
                        .font(egui::TextStyle::Monospace)
                        .hint_text("lno1... (paste your BOLT12 offer here)")
                        .margin(egui::Margin::same(12));

                    ui.add(text_edit);
                });

            ui.add_space(20.0);

            self.decode_offer();

            if let Some(error) = &self.error_message {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(60, 25, 25))
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(15))
                    .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(220, 50, 50)))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("⚠");
                            ui.label(
                                egui::RichText::new(format!("Error: {}", error))
                                    .color(egui::Color32::from_rgb(255, 150, 150)),
                            );
                        });
                    });
                ui.add_space(15.0);
            }

            ui.vertical_centered(|ui| {
                if let Some(offer) = &self.offer {
                    let max_width = 600.0;
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.set_max_width(max_width);
                        egui::Frame::new()
                            .fill(card_bg)
                            .corner_radius(egui::CornerRadius::same(12))
                            .inner_margin(egui::Margin::same(20))
                            .stroke(egui::Stroke::new(2.0, lightning_blue))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("⚡ Decoded Offer Information")
                                            .size(20.0)
                                            .color(lightning_yellow)
                                            .strong(),
                                    );
                                });

                                ui.add_space(15.0);

                                egui::ScrollArea::vertical()
                                    .max_height(400.0)
                                    .show(ui, |ui| {
                                        self.display_offer_field(
                                            ui,
                                            "📝 Description",
                                            offer
                                                .description()
                                                .map(|d| d.to_string())
                                                .unwrap_or_else(|| "No description".to_string()),
                                            lightning_blue,
                                        );

                                        self.display_offer_field(
                                            ui,
                                            "⛓ Chains",
                                            {
                                                let chains = offer.chains();
                                                let chain_strings: Vec<String> = chains
                                                    .into_iter()
                                                    .map(|chain| format!("{:?}", chain))
                                                    .collect();
                                                chain_strings.join(", ")
                                            },
                                            lightning_purple,
                                        );

                                        self.display_offer_field(
                                            ui,
                                            "💰 Amount",
                                            if let Some(amount) = offer.amount() {
                                                match amount {
                                                    Amount::Bitcoin { amount_msats } => {
                                                        format!("{} sats", amount_msats)
                                                    }
                                                    Amount::Currency {
                                                        iso4217_code,
                                                        amount,
                                                    } => format!("{} {:?}", amount, iso4217_code),
                                                }
                                            } else {
                                                "Any amount".to_string()
                                            },
                                            lightning_yellow,
                                        );

                                        self.display_offer_field(
                                            ui,
                                            "🎯 Features",
                                            format!("{:?}", offer.offer_features()),
                                            lightning_blue,
                                        );

                                        self.display_offer_field(
                                            ui,
                                            "🗝 Signing Pubkey",
                                            if let Some(pubkey) = offer.issuer_signing_pubkey() {
                                                format!("{}", pubkey)
                                            } else {
                                                "Not specified".to_string()
                                            },
                                            lightning_purple,
                                        );

                                        self.display_offer_field(
                                            ui,
                                            "🌐 Paths",
                                            format!("{} path(s) available", offer.paths().len()),
                                            lightning_yellow,
                                        );

                                        self.display_offer_field(
                                            ui,
                                            "🏢 Issuer",
                                            if let Some(issuer) = offer.issuer() {
                                                issuer.to_string()
                                            } else {
                                                "Not specified".to_string()
                                            },
                                            lightning_blue,
                                        );

                                        self.display_offer_field(
                                            ui,
                                            "📊 Quantity Max",
                                            match offer.supported_quantity() {
                                                Quantity::Bounded(max) => format!("{:?}", max),
                                                Quantity::Unbounded => "No limit".to_string(),
                                                Quantity::One => "One".to_string(),
                                            },
                                            lightning_purple,
                                        );

                                        self.display_offer_field(
                                            ui,
                                            "⏰ Absolute Expiry",
                                            if let Some(expiry) = offer.absolute_expiry() {
                                                format!("{:?}", expiry)
                                            } else {
                                                "No expiry".to_string()
                                            },
                                            lightning_yellow,
                                        );
                                    });
                            });
                    });
                }

                ui.add_space(30.0);

                ui.label(
                    egui::RichText::new("⚡ Powered by Lightning Network ⚡")
                        .size(12.0)
                        .color(text_secondary)
                        .italics(),
                );
            });
        });
    }
}

impl Bolt12OfferDecoderApp {
    fn decode_offer(&mut self) {
        let trimmed = self.input_text.trim();

        if trimmed.is_empty() {
            self.error_message = None;
            self.offer = None;
            return;
        }

        match Offer::from_str(trimmed) {
            Ok(decoder) => {
                self.offer = Some(decoder);
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to parse offer: {:?}", e));
                self.offer = None;
            }
        }
    }

    fn display_offer_field(
        &self,
        ui: &mut egui::Ui,
        label: &str,
        value: String,
        accent_color: egui::Color32,
    ) {
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(25, 25, 40))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(6))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 70)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(label)
                            .color(accent_color)
                            .strong()
                            .size(14.0),
                    );
                    let value_text = egui::RichText::new(&value)
                        .color(egui::Color32::WHITE)
                        .size(13.0);

                    if value.len() > 60 {
                        ui.label(value_text.family(egui::FontFamily::Monospace));
                    } else {
                        ui.label(value_text);
                    }
                });
            });
    }
}
