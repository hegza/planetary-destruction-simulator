use std::env;
use std::path::Path;
use std::io::prelude::*;
use config::*;
use std::fs;
use serde_yaml;
use util;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub render_cube: bool,
    pub scalar_field_dim: usize,
    pub fixed_fps: f64,
    pub planet_texture: String,
    pub polar_texture: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            render_cube: false,
            scalar_field_dim: 16,
            fixed_fps: 60f64,
            planet_texture: "local_data/planet_textures_by_kakarotti/planet1.jpg".to_owned(),
            polar_texture: "data/texture/ice.png".to_owned(),
        }
    }
}

const CFG_DIR: &str = "data/cfg";
const PROGRAM_CFG: &str = "default.yaml";
const USER_CFG: &str = "user.yaml";
// Full-path to a settings-file can be stored in this environment variable
const CFG_ENV_KEY: &str = "DS_CFG";

lazy_static! {
    static ref DEFAULT: Settings = Settings::default();
}

impl Settings {
    pub fn new() -> Self {
        let mut cfg = Config::new();

        cfg.merge(DEFAULT.clone()).unwrap();

        let per_program_path = format!("{}/{}", CFG_DIR, PROGRAM_CFG);
        // Create the per-program config if it didn't exist
        if !util::file_exists(&per_program_path) {
            let path: &Path = Path::new(&per_program_path);
            let parent: &Path = path.parent().unwrap();
            fs::create_dir_all(parent).unwrap();

            let mut f = fs::File::create(&per_program_path).unwrap();
            f.write_all(&serde_yaml::to_string(&DEFAULT.clone()).unwrap().as_bytes())
                .unwrap();
        }

        // Read and merge the per-program config
        let per_program = File::with_name(&per_program_path);
        cfg.merge(per_program)
            .expect("cannot merge per-program config");

        // Attempt to read the per-user settings
        let per_user_path = format!("{}/{}", CFG_DIR, USER_CFG);
        let per_user_cfg = File::with_name(&per_user_path).required(false);
        cfg.merge(per_user_cfg)
            .expect("cannot merge per-user config");

        // Attempt to read the environment variable settings
        let env_cfg = env::var(CFG_ENV_KEY);
        if let Ok(s) = env_cfg {
            cfg.merge(File::with_name(&format!("{}", s)).required(false))
                .expect("cannot merge env to config");
        }

        cfg.try_into().expect("cannot unwrap config")
    }
}

impl Source for Settings {
    fn clone_into_box(&self) -> Box<Source + Send + Sync> {
        Box::new(self.clone())
    }
    fn collect(&self) -> Result<HashMap<String, Value>, ConfigError> {
        match serde_yaml::from_str(&serde_yaml::to_string(self).unwrap()) {
            Ok(hm) => Ok(hm),
            Err(_e) => panic!(),
        }
    }
}
