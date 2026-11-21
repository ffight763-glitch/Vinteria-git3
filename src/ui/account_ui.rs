//! Account panel: a left panel of its own (apart from the Browser) with the Google profile and the local storage of projects.
use super::{theme::*, VinteriaApp};
use crate::google_auth as g;
use egui::{pos2, vec2, Align2, FontId, RichText, Sense, Stroke};
use std::path::PathBuf;
#[cfg(target_os = "android")]
use crate::rfd;

fn size_text(b: u64) -> String {
    if b >= 1 << 20 { format!("{:.1} MB", b as f64 / (1u64 << 20) as f64) } else { format!("{} KB", (b / 1024).max(1)) }
}

fn card_frame() -> egui::Frame {
    egui::Frame::none().fill(PANEL2()).rounding(card()).stroke(Stroke::new(border_w(), border())).inner_margin(12.0)
}

impl VinteriaApp {
    /// The panel itself; returns true when the user closed it.
    pub(super) fn account_panel(&mut self, ctx: &egui::Context) -> bool {
        let mut close = false;
        let mut open_path: Option<PathBuf> = None;
        let mut open_file = false;
        let mut save = false;
        let mut set_folder: Option<Option<PathBuf>> = None;
        if self.hub_cache.is_none() {
            self.hub_cache = Some(super::hub_projects(&self.recent));
        }
        let list = self.hub_cache.clone().unwrap_or_default();
        egui::SidePanel::left("account_panel").default_width(290.0).min_width(250.0).max_width(380.0)
            .frame(egui::Frame::none().fill(SIDE()).stroke(Stroke::new(border_w(), border())).inner_margin(egui::Margin::symmetric(12.0, 10.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Account").strong().size(15.0).color(TEXT()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("x").on_hover_text("Close the panel").clicked() { close = true; }
                    });
                });
                ui.add_space(8.0);
                egui::ScrollArea::vertical().id_salt("account_scroll").auto_shrink([false, false]).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;
                    ui.set_width(ui.available_width());
                    // ---- profile ----
                    let profile = g::profile();
                    let status = g::status();
                    card_frame().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(vec2(52.0, 52.0), Sense::hover());
                            let initial = profile.as_ref().and_then(|p| p.name.chars().next().or_else(|| p.email.chars().next())).map(|ch| ch.to_uppercase().to_string()).unwrap_or_else(|| "?".into());
                            ui.painter().circle_filled(r.center(), 26.0, if profile.is_some() { TEAL() } else { PANEL3() });
                            ui.painter().text(r.center(), Align2::CENTER_CENTER, initial, FontId::proportional(24.0), if profile.is_some() { egui::Color32::WHITE } else { DIM() });
                            ui.vertical(|ui| {
                                match &profile {
                                    Some(pr) => {
                                        ui.label(RichText::new(if pr.name.is_empty() { "Google account" } else { &pr.name }).strong().size(14.0).color(TEXT()));
                                        ui.label(RichText::new(&pr.email).size(11.0).color(DIM()));
                                    }
                                    None => {
                                        ui.label(RichText::new("Not signed in").strong().size(14.0).color(TEXT()));
                                        ui.label(RichText::new("Projects stay on this computer").size(11.0).color(DIM()));
                                    }
                                }
                            });
                        });
                        ui.add_space(4.0);
                        if profile.is_some() {
                            if ui.button("Sign out").clicked() { g::sign_out(); }
                        } else {
                            if !g::configured() {
                                ui.label(RichText::new("One-time setup: create a \"Desktop app\" OAuth client in Google Cloud Console and put its client_id and client_secret into google_oauth.json next to the program.").size(11.0).color(DIM()));
                                if ui.button("Create google_oauth.json").clicked() {
                                    let path = g::create_config_template();
                                    let _ = std::process::Command::new("notepad").arg(path).spawn();
                                }
                            }
                            match &status {
                                g::Status::Waiting => { ui.horizontal(|ui| { ui.spinner(); ui.label("Waiting for the browser..."); }); }
                                g::Status::Failed(e) => { ui.colored_label(RED, e); }
                                g::Status::SignedOut => {}
                            }
                            if ui.add_enabled(status != g::Status::Waiting, egui::Button::new("Sign in with Google")).clicked() { g::sign_in(); }
                        }
                    });
                    if status == g::Status::Waiting { ui.ctx().request_repaint_after(std::time::Duration::from_millis(300)); }

                    // ---- local storage ----
                    card_frame().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new("Local storage").strong().color(TEXT()));
                        let dir = super::hub_dir();
                        ui.label(RichText::new(dir.display().to_string()).size(10.5).color(DIM()));
                        let total: u64 = list.iter().map(|x| x.2).sum();
                        ui.label(RichText::new(format!("{} projects  -  {}", list.len(), size_text(total))).size(11.0).color(DIM()));
                        ui.horizontal_wrapped(|ui| {
                            if ui.button("Open folder").clicked() { let _ = std::process::Command::new("explorer").arg(&dir).spawn(); }
                            if ui.button("Change...").on_hover_text("Pick another folder for the projects (e.g. a cloud drive folder)").clicked() {
                                if let Some(d) = rfd::FileDialog::new().set_title("Folder for Vinteria Daw projects").pick_folder() {
                                    set_folder = Some(Some(d));
                                }
                            }
                            if ui.button("Default").clicked() { set_folder = Some(None); }
                        });
                        ui.horizontal_wrapped(|ui| {
                            if ui.button("Save").on_hover_text("Save the open project to the folder").clicked() { save = true; }
                            if ui.button("Open file...").clicked() { open_file = true; }
                        });
                    });
                    card_frame().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new("Projects").strong().color(TEXT()));
                        if list.is_empty() { ui.label(RichText::new("No saved projects yet").size(11.0).color(DIM())); }
                        for (path, _, bytes) in list.iter().take(30) {
                            let name = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                            let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::click());
                            if resp.hovered() { ui.painter().rect_filled(r, corner(), PANEL3()); }
                            ui.painter().with_clip_rect(r.shrink2(vec2(0.0, 0.0))).text(r.left_center() + vec2(8.0, 0.0), Align2::LEFT_CENTER, &name, FontId::proportional(13.0), TEXT());
                            ui.painter().text(pos2(r.right() - 8.0, r.center().y), Align2::RIGHT_CENTER, size_text(*bytes), FontId::proportional(10.5), DIM());
                            if resp.on_hover_text(path.display().to_string()).clicked() { open_path = Some(path.clone()); }
                        }
                    });
                });
            });
        if let Some(f) = set_folder {
            super::set_sync_folder(f.as_deref());
            self.hub_cache = None;
        }
        if save { self.save_to_hub(); }
        if open_file { self.open_project(); }
        if let Some(p) = open_path { self.open_path(p); }
        close
    }
}
