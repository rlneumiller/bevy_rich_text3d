use std::sync::{Arc, OnceLock};

use crate::{Text3dPlugin, TextRenderer};
use bevy::{
    ecs::system::{Commands, ResMut, Resource},
    log::error,
    tasks::IoTaskPool,
};
use cosmic_text::fontdb::Database;

#[derive(Debug, Resource)]
pub struct LoadCosmicFonts(pub(crate) Arc<OnceLock<TextRenderer>>);

impl Text3dPlugin {
    pub fn load_fonts_blocking(&self) -> TextRenderer {
        let empty = Database::new();
        let locale = self
            .locale
            .clone()
            .or_else(sys_locale::get_locale)
            .unwrap_or_else(|| "en-US".to_string());
        let mut system = cosmic_text::FontSystem::new_with_locale_and_db(locale, empty);
        if self.load_system_fonts {
            system.db_mut().load_system_fonts();
        }
        for path in &self.load_font_paths {
            if let Err(err) = system.db_mut().load_font_file(path) {
                error!("Error loading font {path}: {err}.");
            };
        }
        for path in &self.load_font_directories {
            system.db_mut().load_fonts_dir(path);
        }
        for data in &self.load_font_embedded {
            system.db_mut().load_font_data(data.to_vec());
        }
        TextRenderer::new(system)
    }

    pub fn load_fonts_concurrent(&self) -> LoadCosmicFonts {
        let locale = self
            .locale
            .clone()
            .or_else(sys_locale::get_locale)
            .unwrap_or_else(|| "en-US".to_string());

        let sender = Arc::new(OnceLock::new());
        let receiver = sender.clone();

        let system_fonts = self.load_system_fonts;
        let font_paths = self.load_font_paths.clone();
        let font_dirs = self.load_font_directories.clone();
        let embedded = self.load_font_embedded.clone();

        IoTaskPool::get()
            .spawn(async move {
                let empty = cosmic_text::fontdb::Database::new();
                let mut system = cosmic_text::FontSystem::new_with_locale_and_db(locale, empty);
                if system_fonts {
                    system.db_mut().load_system_fonts();
                }
                for path in font_paths {
                    if let Err(err) = system.db_mut().load_font_file(&path) {
                        error!("Error loading font {path}: {err}.");
                    };
                }
                for path in font_dirs {
                    system.db_mut().load_fonts_dir(path);
                }
                for data in embedded {
                    system.db_mut().load_font_data(data.to_vec());
                }
                sender.set(TextRenderer::new(system))
            })
            .detach();
        LoadCosmicFonts(receiver)
    }
}

pub fn load_cosmic_fonts_system(mut commands: Commands, mut load: ResMut<LoadCosmicFonts>) {
    if let Some(system) = Arc::get_mut(&mut load.0) {
        if let Some(system) = system.take() {
            commands.insert_resource(system);
            commands.remove_resource::<LoadCosmicFonts>();
        }
    }
}
