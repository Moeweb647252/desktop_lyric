use eframe::egui::{
    CentralPanel, Context, Frame, Label, Margin, PointerButton, RichText, Rounding, Sense, Vec2,
    ViewportCommand,
};
use log::{debug, info};

use super::DesktopLyricApp;

impl DesktopLyricApp {
    pub fn lyric_ui(&mut self, ctx: &Context) {
        let resp = CentralPanel::default()
            .frame(Frame {
                fill: self.config.background_color.color(),
                rounding: Rounding::same(10.0),
                inner_margin: Margin::symmetric(10.0, 5.0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                let cur_lyric = { self.current_lyric.read().clone() };
                let resp = if self.config.auto_resize {
                    ui.add(
                        Label::new(
                            RichText::new(format!("{}", &cur_lyric))
                                .color(self.config.text_color.color())
                                .size(self.config.text_size),
                        )
                        .extend(),
                    )
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.add(
                            Label::new(
                                RichText::new(format!("{}", &cur_lyric))
                                    .color(self.config.text_color.color())
                                    .size(self.config.text_size),
                            )
                            .extend(),
                        )
                    })
                    .inner
                };

                let screen_rect = ctx.input(|v| v.screen_rect().max);
                if (resp.rect.max.x, resp.rect.max.y)
                    != (screen_rect.x.round() - 20.0, screen_rect.y.round() - 10.0)
                    && self.config.auto_resize
                {
                    debug!(
                        "Container size: {:?}, Screen size: {:?}",
                        resp.rect.max, screen_rect
                    );
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::new(
                        resp.rect.max.x + 20.0,
                        resp.rect.max.y + 10.0,
                    )));
                }
            })
            .response
            .interact(Sense::click_and_drag());
        if resp.clicked_by(PointerButton::Secondary) {
            self.drag_mode = !self.drag_mode;
            info!("Drag mode: {}", self.drag_mode)
        }

        if self.drag_mode {
            if resp.drag_started() {
                ctx.send_viewport_cmd(ViewportCommand::StartDrag);
            }
        } else {
            if self.config.auto_resize {
                if resp.dragged() {
                    if resp.drag_delta().y > 0.0 {
                        self.config.text_size += 1.0;
                    } else if resp.drag_delta().y < 0.0 {
                        self.config.text_size -= 1.0;
                    }
                }
            } else {
                if resp.drag_started() {
                    info!("Begin resize");
                    ctx.send_viewport_cmd(ViewportCommand::BeginResize(
                        eframe::egui::ResizeDirection::SouthEast,
                    ));
                }
            }
        }
    }
}
