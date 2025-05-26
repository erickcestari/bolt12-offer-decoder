use eframe::egui;
use lightning::offers::offer::{Offer, Quantity};
use std::str::FromStr;

pub struct Bolt12OfferDecoderApp {
    pub offer: Option<Offer>,
    input_text: String,
    error_message: Option<String>,
}

impl Default for Bolt12OfferDecoderApp {
    fn default() -> Self {
        Self {
            offer: None,
            input_text: String::new(),
            error_message: None,
        }
    }
}

impl eframe::App for Bolt12OfferDecoderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("BOLT12 Offer Decoder");

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Enter BOLT12 Offer:");
            });

            let text_edit_response = ui.add(
                egui::TextEdit::multiline(&mut self.input_text)
                    .desired_width(f32::INFINITY)
                    .desired_rows(4)
                    .font(egui::TextStyle::Monospace),
            );

            ui.add_space(10.0);

            if ui.button("Decode Offer").clicked()
                || (text_edit_response.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                self.decode_offer();
            }

            if ui.button("Clear").clicked() {
                self.input_text.clear();
                self.offer = None;
                self.error_message = None;
            }

            ui.separator();

            // Display error message if any
            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                ui.add_space(10.0);
            }

            // Display decoded offer information
            if let Some(offer) = &self.offer {
                ui.heading("Decoded Offer Information:");
                ui.add_space(5.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("offer_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            // Description
                            ui.label("Description:");
                            ui.label(
                                offer
                                    .description()
                                    .map(|d| d.to_string())
                                    .unwrap_or_else(|| "No description".to_string()),
                            );
                            ui.end_row();

                            // Chains
                            ui.label("Chains:");
                            let chains = offer.chains();
                            let chain_strings: Vec<String> = chains
                                .into_iter()
                                .map(|chain| format!("{:?}", chain))
                                .collect();
                            ui.label(chain_strings.join(", "));
                            ui.end_row();

                            // Amount
                            ui.label("Amount:");
                            if let Some(amount) = offer.amount() {
                                ui.label(format!("{:?}", amount));
                            } else {
                                ui.label("Any amount");
                            }
                            ui.end_row();

                            // Features
                            ui.label("Features:");
                            ui.label(format!("{:?}", offer.offer_features()));
                            ui.end_row();

                            // Signing Pubkey
                            ui.label("Signing Pubkey:");
                            if let Some(pubkey) = offer.issuer_signing_pubkey() {
                                ui.label(format!("{:?}", pubkey));
                            } else {
                                ui.label("Not specified");
                            }
                            ui.end_row();

                            // Paths
                            ui.label("Paths:");
                            let paths = offer.paths();
                            let path_count = paths.len();
                            ui.label(format!("{} path(s) available", path_count));
                            ui.end_row();

                            // Issuer
                            ui.label("Issuer:");
                            if let Some(issuer) = offer.issuer() {
                                ui.label(issuer.to_string());
                            } else {
                                ui.label("Not specified");
                            }
                            ui.end_row();

                            // Quantity Max
                            ui.label("Quantity Max:");
                            match offer.supported_quantity() {
                                Quantity::Bounded(max) => {
                                    ui.label(format!("{:?}", max));
                                }
                                Quantity::Unbounded => {
                                    ui.label("No limit");
                                }
                                Quantity::One => {
                                    ui.label("One");
                                }
                            };
                            ui.end_row();

                            // Expiry
                            ui.label("Absolute Expiry:");
                            if let Some(expiry) = offer.absolute_expiry() {
                                ui.label(format!("{:?}", expiry));
                            } else {
                                ui.label("No expiry");
                            }
                            ui.end_row();
                        });
                });
            }
        });
    }
}

impl Bolt12OfferDecoderApp {
    fn decode_offer(&mut self) {
        let trimmed = self.input_text.trim();

        if trimmed.is_empty() {
            self.error_message = Some("Please enter a BOLT12 offer".to_string());
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
}
