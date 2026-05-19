use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModeConfig {
    pub theme_idx: usize,
    pub charset_idx: usize,
}
#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub last_mode: String,
    pub modes: HashMap<String, ModeConfig>,
    #[serde(default)]
    pub user_name: Option<String>,
    #[serde(default)]
    pub utc_offset_hours: i32,
}
impl Default for AppConfig {
    fn default() -> Self {
        let mut modes = HashMap::new();
        let config_data = [
            ("Maze", 7, 4),
            ("RaceTrack", 7, 0),
            ("Breaker", 2, 0),
            ("SpinningName", 4, 1),
            ("Life", 3, 4),
            ("Cube", 7, 5),
            ("Rain", 7, 6),
            ("Synthwave", 4, 6),
            ("Name", 2, 4),
            ("Minecraft", 0, 2),
            ("Stripes", 1, 0),
            ("Tetris", 2, 4),
            ("Waves", 7, 3),
            ("Snake", 5, 4),
            ("Matrix", 8, 0),
            ("Earth", 0, 2),
            ("Lorenz", 2, 6),
            ("Perlin", 3, 2),
            ("PingPong", 9, 0),
            ("Shape", 7, 0),
            ("Bubbles", 7, 4),
            ("Pacman", 6, 2),
            ("Sand", 6, 6),
            ("Metaballs", 4, 6),
            ("Fire", 1, 4),
            ("Hex3D", 3, 6),
            ("Pipes", 8, 0),
            ("Stars", 9, 6),
            ("City3D", 3, 6),
            ("Logos", 8, 0),
            ("Bonsai", 9, 0),
            ("DVD", 3, 0),
            ("Boids", 3, 0),
            ("Clocks", 0, 0),
        ];
        for (name, theme_idx, charset_idx) in config_data {
            modes.insert(
                name.to_string(),
                ModeConfig {
                    theme_idx,
                    charset_idx,
                },
            );
        }
        Self {
            last_mode: "Matrix".to_string(),
            modes,
            user_name: None,
            utc_offset_hours: 0,
        }
    }
}
impl AppConfig {
    pub fn load() -> Self {
        let mut default_config = Self::default();
        if let Ok(data) = fs::read_to_string("config.json") {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&data) {
                if !config.last_mode.is_empty() {
                    default_config.last_mode = config.last_mode;
                }
                for (k, v) in config.modes {
                    default_config.modes.insert(k, v);
                }
                default_config.user_name = config.user_name;
            }
        }
        if default_config.user_name.is_none()
            || default_config
                .user_name
                .as_ref()
                .map_or(true, |s| s.is_empty())
        {
            let env_username = std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .unwrap_or_else(|_| "User".to_string());
            default_config.user_name = Some(env_username);
        }
        default_config.save();
        default_config
    }
    pub fn save(&self) {
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = fs::write("config.json", data);
        }
    }
    pub fn get_mode_config(&self, mode_name: &str) -> ModeConfig {
        self.modes.get(mode_name).cloned().unwrap_or(ModeConfig {
            theme_idx: 0,
            charset_idx: 0,
        })
    }
    pub fn set_mode_config(&mut self, mode_name: &str, theme_idx: usize, charset_idx: usize) {
        self.modes.insert(
            mode_name.to_string(),
            ModeConfig {
                theme_idx,
                charset_idx,
            },
        );
        self.save();
    }
}
