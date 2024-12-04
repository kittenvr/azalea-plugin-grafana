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

    pub fn update_player_count(&self, count: i64) {
        self.player_count.set(count);
    }

    pub fn update_tps(&self, tps: i64) {
        self.tps.set(tps);
    }

    pub fn update_latency(&self, latency: i64) {
        self.latency.set(latency);
    }
}
