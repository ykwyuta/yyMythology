use super::*;

impl Game {
    pub(super) fn render_god_catalog(&mut self) {
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        let pw = 640.0f32.min(w - 20.0);
        let x = (w - pw) * 0.5;
        let y = (h - 330.0) * 0.5;
        let group = self.chron_scroll;
        let pl = self.world.player;
        let ours = group == pantheon(pl);
        let names = PANTHEONS[group].name;
        let count = self.world.god_count(group);
        let heading = format!("{}   {}/{}", t!(names[0], names[1], names[2]), count, GOD_LIMIT);
        let own = self.own_sel();
        let can_sacrifice = !self.world.sacrifice_power(pl, &own).0.is_empty();
        let tip = t!("資源と自国領内の民1人を捧げ、都に召喚する。", "Spend resources and sacrifice one worker in your territory to summon at your capital.", "消耗资源并献祭一名己方领土内的民，在都城召唤。");
        let gold = [1.0, 0.85, 0.5, 1.0];

        self.gfx.fill_rect(0.0, 0.0, w, h, [0.0, 0.0, 0.0, 0.55]);
        self.art.draw_stone_panel(&self.gfx, x, y, x + pw, y + 330.0);
        self.gfx.text_shadow(t!("神名辞典", "Divine Codex", "神名辞典"), x + 20.0, y + 12.0, pw - 136.0, 30.0, Font::Big, gold);
        self.add_button(x + pw - 108.0, y + 8.0, x + pw - 16.0, y + 42.0, t!("閉じる", "Close", "关闭"), "", 0, true, Act::CloseOverlay, "");
        self.gfx.text(&heading, x + 22.0, y + 61.0, pw - 44.0, 28.0, Font::Big, gold);

        for i in 0..GOD_LIMIT {
            let god = (group * GOD_LIMIT + i) as u8;
            let row = y + 100.0 + i as f32 * 53.0;
            let name = god_name(god).unwrap();
            let present = self.world.units.iter().any(|u| u.alive && u.god == god);
            if let Some(portrait) = self.art.summoned_god(god, false) {
                self.gfx.draw_bitmap(portrait, x + 19.0, row - 3.0, x + 66.0, row + 44.0);
            }
            self.gfx.text(name, x + 73.0, row + 8.0, pw * 0.40, 24.0, Font::Normal, gold);
            if present {
                self.gfx.text(t!("召喚中", "Summoned", "已召唤"), x + pw - 200.0, row + 8.0, 180.0, 24.0, Font::Small, gold);
            } else if ours {
                let ok = self.world.can_summon_god(pl, god, &own);
                self.add_button(x + pw - 200.0, row, x + pw - 18.0, row + 42.0,
                    t!("召喚", "Summon", "召唤"), &Self::cost_str(GOD_COST), 0, ok, Act::SummonGod(god), tip);
            }
        }
        if ours && !can_sacrifice {
            self.gfx.text(t!("召喚するには、自国領内の民を選んでから辞典を開く。", "Select a worker in your territory before opening the Codex to summon.", "请先选中己方领土内的民，再打开神名辞典召唤。"),
                x + 20.0, y + 250.0, pw - 40.0, 23.0, Font::Small, gold);
        }
        let by = y + 275.0;
        self.add_button(x + 20.0, by, x + 126.0, by + 38.0, "◀", "", 0, group > 0, Act::ChronPage(-1), "");
        self.add_button(x + pw - 126.0, by, x + pw - 20.0, by + 38.0, "▶", "", 0, group + 1 < PANTHEONS.len(), Act::ChronPage(1), "");
        self.gfx.text(&format!("{} / {}", group + 1, PANTHEONS.len()), x + 130.0, by + 7.0, pw - 260.0, 24.0, Font::NormalCenter, gold);
    }
}
