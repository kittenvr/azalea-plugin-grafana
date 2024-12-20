use prometheus::{IntGauge, register_int_gauge};

pub struct Metrics {
    pub player_count: IntGauge,
    pub tps: IntGauge,
    pub latency: IntGauge,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            player_count: register_int_gauge!("minecraft_player_count", "Number of players online").unwrap(),
            tps: register_int_gauge!("minecraft_tps", "Server TPS").unwrap(),
            latency: register_int_gauge!("minecraft_latency", "Bot latency").unwrap(),
        }
    }

    pub fn update(&self, player_count: i64, tps: i64, latency: i64) {
        self.player_count.set(player_count);
        self.tps.set(tps);
        self.latency.set(latency);
    }
}
