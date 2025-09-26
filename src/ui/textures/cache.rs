use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
use eframe::egui::Sense;
use eframe::egui_wgpu::RenderState;
use eframe::epaint::TextureId;
use eframe::epaint::mutex::RwLock;
use eframe::epaint::vec2;
use eframe::wgpu;
use gwynn_texture::Texture;
use linked_hash_map::LinkedHashMap;
use std::rc::Rc;
use std::sync::Arc;
use uuid::Uuid;

pub type LoadedTexture = (Arc<Texture>, TextureId);
pub(crate) type TextureCacheMap = LinkedHashMap<Uuid, CachedTexture>;

pub enum CachedTexture {
    Loading,
    Loaded(Option<LoadedTexture>),
}

#[derive(Clone)]
pub struct TextureCache {
    pub render_state: RenderState,
    pub(crate) cache: Rc<RwLock<TextureCacheMap>>,
    // pub(crate) loading_placeholder: LoadedTexture,
    loader: Rc<TextureLoader>,
}

impl TextureCache {
    pub fn new(render_state: RenderState) -> Self {
        // let loading_placeholder =
        //     Texture::load_png(&render_state, include_bytes!("../../loading.png")).unwrap();

        // let loading_placeholder_id = render_state.renderer.write().register_native_texture(
        //     &render_state.device,
        //     &loading_placeholder.view,
        //     wgpu::FilterMode::Linear,
        // );

        Self {
            loader: Rc::new(TextureLoader::new(2, render_state.clone())),
            render_state,
            cache: Rc::new(RwLock::new(TextureCacheMap::default())),
            // loading_placeholder: (Arc::new(loading_placeholder), loading_placeholder_id),
        }
    }

    pub fn is_loading_textures(&self) -> bool {
        self.cache
            .read()
            .iter()
            .any(|(_, v)| matches!(v, CachedTexture::Loading))
    }

    // pub fn get_or_default(&self, hash: Uuid) -> LoadedTexture {
    //     self.get_or_load(hash)
    //         .unwrap_or_else(|| self.loading_placeholder.clone())
    // }

    pub fn get_or_load(&self, hash: Uuid) -> Option<LoadedTexture> {
        self.maintain();

        let mut cache = self.cache.write();
        let c = cache.get(&hash);
        if let Some(CachedTexture::Loaded(r)) = c {
            r.clone()
        } else if c.is_none() {
            cache.insert(hash, CachedTexture::Loading);
            self.loader.request_texture_load(hash);
            None
        } else {
            None
        }
    }

    fn maintain(&self) {
        let textures = self.loader.receive_textures();
        if !textures.is_empty() {
            let mut cache = self.cache.write();
            for (tex, hash) in textures {
                if let Some(CachedTexture::Loading) = cache.get(&hash) {
                    let id = self.render_state.renderer.write().register_native_texture(
                        &self.render_state.device,
                        &tex.view,
                        wgpu::FilterMode::Linear,
                    );
                    cache.insert(hash, CachedTexture::Loaded(Some((tex, id))));
                }
            }
        }

        self.truncate();
    }

    // pub(crate) async fn load_texture_task(
    //     render_state: RenderState,
    //     hash: TagHash,
    // ) -> Option<LoadedTexture> {
    //     let texture = match Texture::load(&render_state, hash, true) {
    //         Ok(t) => t,
    //         Err(e) => {
    //             log::error!("Failed to load texture {hash}: {e}");
    //             return None;
    //         }
    //     };

    //     let id = render_state.renderer.write().register_native_texture(
    //         &render_state.device,
    //         &texture.view,
    //         wgpu::FilterMode::Linear,
    //     );
    //     Some((Arc::new(texture), id))
    // }

    pub fn texture_preview(&self, hash: Uuid, ui: &mut eframe::egui::Ui) {
        if let Some((tex, egui_tex)) = self.get_or_load(hash) {
            let screen_size = ui.ctx().screen_rect().size();
            let screen_aspect_ratio = screen_size.x / screen_size.y;
            let texture_aspect_ratio = tex.aspect_ratio();

            let max_size = if ui.input(|i| i.modifiers.ctrl) {
                screen_size * 0.70
            } else {
                ui.label("ℹ Hold ctrl to enlarge");
                screen_size * 0.30
            };

            let tex_size = if texture_aspect_ratio > screen_aspect_ratio {
                vec2(max_size.x, max_size.x / texture_aspect_ratio)
            } else {
                vec2(max_size.y * texture_aspect_ratio, max_size.y)
            };

            let (response, painter) = ui.allocate_painter(tex_size, Sense::hover());
            // ui_image_rotated(
            //     &painter,
            //     egui_tex,
            //     response.rect,
            //     // Rotate the image if it's a cubemap
            //     if tex.desc.kind() == TextureType::TextureCube {
            //         90.
            //     } else {
            //         0.
            //     },
            //     tex.desc.kind() == TextureType::TextureCube,
            // );

            // ui.horizontal(|ui| {
            //     match tex.desc.kind() {
            //         TextureType::Texture2D => ui.chip("2D", Color32::YELLOW, Color32::BLACK),
            //         TextureType::TextureCube => ui.chip("Cube", Color32::BLUE, Color32::WHITE),
            //         TextureType::Texture3D => ui.chip("3D", Color32::GREEN, Color32::BLACK),
            //     };

            //     ui.label(tex.desc.info());
            // });
        }
    }

    pub(crate) const MAX_TEXTURES: usize = 2048;
    pub(crate) fn truncate(&self) {
        let mut cache = self.cache.write();
        while cache.len() > Self::MAX_TEXTURES {
            if let Some((_, CachedTexture::Loaded(Some((_, tid))))) = cache.pop_front() {
                self.render_state.renderer.write().free_texture(&tid);
            }
        }
    }
}

struct TextureLoader {
    request_tx: Sender<Uuid>,
    texture_rx: Receiver<(Arc<Texture>, Uuid)>,
    _threads: Vec<std::thread::JoinHandle<()>>,
}

impl TextureLoader {
    pub fn new(thread_count: usize, render_state: RenderState) -> Self {
        let (request_tx, request_rx) = crossbeam::channel::unbounded();
        let (texture_tx, texture_rx) = crossbeam::channel::unbounded();

        let mut threads = vec![];
        for i in 0..thread_count {
            let rx = request_rx.clone();
            let tx = texture_tx.clone();
            let render_state = render_state.clone();
            threads.push(
                std::thread::Builder::new()
                    .name(format!("texture_loader_{i}"))
                    .spawn(move || texture_loader_thread(render_state, rx, tx))
                    .expect("Failed to spawn texture loader thread"),
            );
        }

        Self {
            request_tx,
            texture_rx,
            _threads: threads,
        }
    }

    pub fn request_texture_load(&self, hash: Uuid) {
        let _ = self.request_tx.send(hash);
    }

    pub fn receive_textures(&self) -> Vec<(Arc<Texture>, Uuid)> {
        let mut textures = vec![];
        while let Ok(t) = self.texture_rx.try_recv() {
            textures.push(t);
        }
        textures
    }
}

fn texture_loader_thread(
    render_state: RenderState,
    rx: Receiver<Uuid>,
    tx: Sender<(Arc<Texture>, Uuid)>,
) {
    while let Ok(hash) = rx.recv() {
        // let _ = Self::load_texture_task(render_state.clone(), hash);
    }
}
