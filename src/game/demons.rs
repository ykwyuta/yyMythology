use super::*;

impl Game {
    pub(super) fn render_demons(&mut self) {
        let w = self.gfx.width as f32;
        let h = self.gfx.height as f32;
        for demon in &self.world.demons {
            let (x, y) = self.world_to_screen(demon.pos);
            if x < -64.0 || x > w + 64.0 || y < -64.0 || y > h + 64.0 {
                continue;
            }
            let color = demon.kind.color(1.0);
            if self.zoom < 3.0 {
                self.gfx.fill_circle(x, y, 3.5, color);
                continue;
            }

            let radius = (self.zoom * 1.5).clamp(13.0, 40.0);
            self.gfx.fill_circle(x, y + 3.0, radius * 0.35, demon.kind.color(0.24));
            if let Some(bitmap) = self.art.demon_sprite(demon.kind) {
                self.gfx.draw_bitmap(bitmap, x - radius, y - radius * 1.65, x + radius, y + radius * 0.35);
            }
            let name = demon.kind.name();
            let tag = tf!("{}・無敵", "{} · Invulnerable", "{}・无敌", name);
            let tw = self.gfx.measure(&tag, Font::NormalCenter) + 14.0;
            let top = y - radius * 1.68 - 20.0;
            self.gfx.fill_round(x - tw * 0.5, top, x + tw * 0.5, top + 20.0, 3.0, [0.07, 0.07, 0.10, 0.92]);
            self.gfx.stroke_round(x - tw * 0.5, top, x + tw * 0.5, top + 20.0, 3.0, 1.2, color);
            self.gfx.text(&tag, x - tw * 0.5, top + 1.0, tw, 18.0, Font::NormalCenter, color);
        }
    }
}
