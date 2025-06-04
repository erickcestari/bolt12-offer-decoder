use eframe::egui;
use lightning::{
    bitcoin::bech32::{NoChecksum, primitives::decode::CheckedHrpstring},
    offers::{
        offer::{Amount, Offer, Quantity},
        parse::Bolt12ParseError,
    },
};
use std::str::FromStr;

struct Theme {
    accent_color: egui::Color32,
    secondary_accent_color: egui::Color32,
    text_color: egui::Color32,
    text_color_secondary: egui::Color32,
    card_bg: egui::Color32,
    dark_bg: egui::Color32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent_color: egui::Color32::from_rgb(30, 144, 255),
            secondary_accent_color: egui::Color32::from_rgb(255, 215, 0),
            text_color: egui::Color32::from_rgb(160, 160, 180),
            text_color_secondary: egui::Color32::from_rgb(138, 43, 226),
            card_bg: egui::Color32::from_rgb(35, 35, 50),
            dark_bg: egui::Color32::from_rgb(25, 25, 35),
        }
    }
}

const BECH32_HRP: &'static str = "lno";

// Used to avoid copying a bech32 string not containing the continuation character (+).
enum Bech32String<'a> {
    Borrowed(&'a str),
    Owned(String),
}

impl<'a> AsRef<str> for Bech32String<'a> {
    fn as_ref(&self) -> &str {
        match self {
            Bech32String::Borrowed(s) => s,
            Bech32String::Owned(s) => s,
        }
    }
}

fn from_bech32_str(s: &str) -> Result<Vec<u8>, Bolt12ParseError> {
    // Offer encoding may be split by '+' followed by optional whitespace.
    let encoded = match s.split('+').skip(1).next() {
        Some(_) => {
            for chunk in s.split('+') {
                let chunk = chunk.trim_start();
                if chunk.is_empty() || chunk.contains(char::is_whitespace) {
                    return Err(Bolt12ParseError::InvalidContinuation);
                }
            }

            let s: String = s
                .chars()
                .filter(|c| *c != '+' && !c.is_whitespace())
                .collect();
            Bech32String::Owned(s)
        }
        None => Bech32String::Borrowed(s),
    };

    let parsed = CheckedHrpstring::new::<NoChecksum>(encoded.as_ref())?;
    let hrp = parsed.hrp();
    // Compare the lowercase'd iter to allow for all-uppercase HRPs
    if hrp.lowercase_char_iter().ne(BECH32_HRP.chars()) {
        return Err(Bolt12ParseError::InvalidBech32Hrp);
    }

    let data = parsed.byte_iter().collect::<Vec<u8>>();
    Ok(data)
}

pub struct Bolt12OfferDecoderApp {
    pub offer: Option<Offer>,
    input_text: String,
    error_message: Option<String>,
    theme: Theme,
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
            theme: Theme::default(),
        }
    }
}

impl eframe::App for Bolt12OfferDecoderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.decode_offer();

        let mut style = (*ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.panel_fill = self.theme.dark_bg;
        style.visuals.extreme_bg_color = self.theme.card_bg;
        style.visuals.widgets.noninteractive.bg_fill = self.theme.card_bg;
        style.visuals.widgets.inactive.bg_fill = self.theme.card_bg;
        style.visuals.widgets.active.bg_fill = self.theme.accent_color;
        style.visuals.selection.bg_fill = self.theme.accent_color;
        style.spacing.item_spacing = egui::vec2(12.0, 8.0);
        style.spacing.button_padding = egui::vec2(16.0, 8.0);
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);
            self.display_title(ui);
            self.display_text_field(ui);

            ui.add_space(20.0);
            match from_bech32_str(&self.input_text) {
                Ok(data_part) => {
                    let data_part_str = data_part
                        .iter()
                        .map(|d| format!("{:02x}", d))
                        .collect::<Vec<_>>()
                        .join("");
                    let title = egui::RichText::new(format!("data part: {}", data_part_str))
                        .size(12.0)
                        .color(self.theme.secondary_accent_color)
                        .strong();
                    ui.label(title);
                }
                Err(_) => {}
            }
            if let Some(offer) = &self.offer {
                self.display_offer(ui, offer);
            }

            if let Some(error) = &self.error_message {
                self.display_error(ui, error);
            }

            ui.add_space(30.0);
            self.display_footer(ui);
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

    fn display_offer_paths(&self, ui: &mut egui::Ui, offer: &Offer) {
        let paths = offer.paths();
        let path_strings: Vec<String> = paths
            .into_iter()
            .map(|path| format!("{:?}", path))
            .collect();
        self.display_offer_field(
            ui,
            "🌐 Paths",
            path_strings.join(", "),
            self.theme.text_color,
        );
    }

    fn display_offer_field(
        &self,
        ui: &mut egui::Ui,
        label: &str,
        value: String,
        accent_color: egui::Color32,
    ) {
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
            ui.label(value_text);
        });
    }

    fn display_error(&self, ui: &mut egui::Ui, error: &str) {
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

    fn display_text_field(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(self.theme.card_bg)
            .corner_radius(egui::CornerRadius::same(12))
            .inner_margin(egui::Margin::same(20))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 80)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("🔍 Enter BOLT12 Offer:")
                            .size(16.0)
                            .color(self.theme.accent_color)
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
    }

    fn display_title(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            let title = egui::RichText::new("⚡ BOLT12 Offer Decoder ⚡")
                .size(32.0)
                .color(self.theme.secondary_accent_color)
                .strong();
            ui.label(title);

            let subtitle = egui::RichText::new("Lightning Network Offer Analysis Tool")
                .size(14.0)
                .color(self.theme.text_color)
                .italics();
            ui.label(subtitle);
            ui.add_space(15.0);
        });
    }

    fn display_footer(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(30.0);
            ui.label(
                egui::RichText::new("⚡ Powered by Lightning Network ⚡")
                    .size(12.0)
                    .color(self.theme.text_color)
                    .italics(),
            );
        });
    }

    fn display_offer(&self, ui: &mut egui::Ui, offer: &Offer) {
        let max_width = 600.0;
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.set_max_width(max_width);
            egui::Frame::new()
                .fill(self.theme.card_bg)
                .corner_radius(egui::CornerRadius::same(12))
                .inner_margin(egui::Margin::same(20))
                .stroke(egui::Stroke::new(2.0, self.theme.accent_color))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("⚡ Decoded Offer Information")
                                .size(20.0)
                                .color(self.theme.secondary_accent_color)
                                .strong(),
                        );
                    });

                    ui.add_space(15.0);

                    egui::ScrollArea::vertical()
                        .max_height(500.0)
                        .show(ui, |ui| {
                            self.display_offer_field(
                                ui,
                                "📝 Description",
                                offer
                                    .description()
                                    .map(|d| d.to_string())
                                    .unwrap_or_else(|| "No description".to_string()),
                                self.theme.accent_color,
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
                                self.theme.text_color_secondary,
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
                                        } => {
                                            let code_str = match std::str::from_utf8(&iso4217_code)
                                            {
                                                Ok(s) => s,
                                                Err(_) => "Unknown",
                                            };
                                            format!("{} {}", amount, code_str)
                                        }
                                    }
                                } else {
                                    "Any amount".to_string()
                                },
                                self.theme.secondary_accent_color,
                            );

                            self.display_offer_field(
                                ui,
                                "🎯 Features",
                                format!("{:?}", offer.offer_features()),
                                self.theme.accent_color,
                            );

                            self.display_offer_field(
                                ui,
                                "🗝 Signing Pubkey",
                                if let Some(pubkey) = offer.issuer_signing_pubkey() {
                                    format!("{}", pubkey)
                                } else {
                                    "Not specified".to_string()
                                },
                                self.theme.text_color_secondary,
                            );

                            self.display_offer_paths(ui, offer);

                            self.display_offer_field(
                                ui,
                                "🏢 Issuer",
                                if let Some(issuer) = offer.issuer() {
                                    issuer.to_string()
                                } else {
                                    "Not specified".to_string()
                                },
                                self.theme.accent_color,
                            );

                            self.display_offer_field(
                                ui,
                                "📊 Quantity Max",
                                match offer.supported_quantity() {
                                    Quantity::Bounded(max) => format!("{:?}", max),
                                    Quantity::Unbounded => "No limit".to_string(),
                                    Quantity::One => "One".to_string(),
                                },
                                self.theme.text_color_secondary,
                            );

                            self.display_offer_field(
                                ui,
                                "⏰ Absolute Expiry",
                                if let Some(expiry) = offer.absolute_expiry() {
                                    format!("{:?}", expiry)
                                } else {
                                    "No expiry".to_string()
                                },
                                self.theme.secondary_accent_color,
                            );
                        });
                });
        });
    }
}
