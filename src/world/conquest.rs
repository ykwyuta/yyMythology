//! 征服にまつわる出来事：邑の破壊（焼き払い）、服属（併合）、新しい国の成立、
//! 靖康の変（宋が南宋に）、神風（元寇の船が沈む）

use super::*;

impl World {
    /// 邑を破壊する（焼き払う）。都は破壊できない。戦利品として資源を得る
    pub fn can_raze(&self, bi: usize, by: usize) -> Result<(), &'static str> {
        let b = &self.blds[bi];
        if !b.alive || !b.kind.city() || b.nation as usize != by {
            return Err(t!("自国の邑だけを破壊できる", "You can only raze your own towns", "只能摧毁己方的邑"));
        }
        if b.kind == BKind::Capital {
            return Err(t!("都は破壊できない", "The capital cannot be razed", "都城无法摧毁"));
        }
        if !b.done() {
            return Err(t!("建設中の邑は取り壊す", "Demolish a town still under construction instead", "建造中的邑请直接拆除"));
        }
        Ok(())
    }

    /// 邑を焼き払う。戦利品（糧・木・鉄）を返す
    pub fn raze_city(&mut self, bi: usize, by: usize) -> [f32; 3] {
        if self.can_raze(bi, by).is_err() {
            return [0.0; 3];
        }
        let origin = self.blds[bi].origin as usize;
        let place = self.city_place(bi);
        let m = self.era_mult();
        // 他国が築いた邑ほど多く奪える
        let k = if origin != by { 1.0 } else { 0.5 };
        let loot = [160.0 * m * k, 110.0 * m * k, 70.0 * m * k];
        self.remove_building(bi);
        if by < NATIONS.len() {
            let n = &mut self.nations[by];
            for r in 0..3 {
                n.res[r] += loot[r];
            }
            n.aggression += 15.0;
        }
        let who = tx(self.nations[by].name);
        let text = tf!("{} が{}の邑を焼き払った（糧{:.0}・木{:.0}・鉄{:.0}を得た）", "{} razed the town at {} (took food {:.0}, wood {:.0}, iron {:.0})", "{}焚毁了{}的邑（获得粮{:.0}、木{:.0}、铁{:.0}）", who, place, loot[0], loot[1], loot[2]);
        let col = if by == self.player { [1.0, 0.75, 0.4] } else { [0.85, 0.7, 0.6] };
        self.push_msg(text, col);
        self.territory_dirty = true;
        loot
    }

    /// AI：攻め落とした邑を焼き払うか（遊牧の国ほど焼く。城邑が少なければ焼かない）
    pub(super) fn ai_wants_raze(&mut self, an: usize, bi: usize) -> bool {
        let cities = self.blds.iter().filter(|b| b.alive && b.nation as usize == an && b.kind.city() && b.done()).count();
        if cities < 4 || self.blds[bi].origin as usize == an {
            return false;
        }
        let p = match an {
            MONGOL => 0.3,
            NAIMAN | KERAIT | TATAR | MERKIT => 0.15,
            _ => 0.04,
        };
        self.rng.chance(p)
    }
}
