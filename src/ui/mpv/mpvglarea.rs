use glib::Object;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use crate::client::client::EMBY_CLIENT;
use crate::client::structs::Back;

use super::tsukimi_mpv::{MpvTrack, TrackSelection, ACTIVE};

mod imp {
    use gtk::gdk::GLContext;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::ui::mpv::tsukimi_mpv::TsukimiMPV;
    use crate::ui::mpv::tsukimi_mpv::RENDER_UPDATE;
    use tracing::error;

    // Object holding the state
    #[derive(Default)]
    pub struct MPVGLArea {
        pub mpv: TsukimiMPV,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for MPVGLArea {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "MPVGLArea";
        type Type = super::MPVGLArea;
        type ParentType = gtk::GLArea;
    }

    impl ObjectImpl for MPVGLArea {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for MPVGLArea {
        fn realize(&self) {
            self.parent_realize();
            let obj = self.obj();
            obj.make_current();
            let Some(gl_context) = self.obj().context() else {
                error!("Failed to get GLContext");
                return;
            };

            self.mpv.connect_render_update(gl_context);

            glib::spawn_future_local(glib::clone!(
                #[weak]
                obj,
                async move {
                    while let Ok(true) = RENDER_UPDATE.rx.recv_async().await {
                        obj.queue_render();
                    }
                }
            ));

            self.mpv.process_events();
        }
    }

    impl GLAreaImpl for MPVGLArea {
        fn render(&self, _context: &GLContext) -> glib::Propagation {
            if let Some(ctx) = self.mpv.ctx.borrow().as_ref() {
                let factor = self.obj().scale_factor();
                let width = self.obj().width() * factor;
                let height = self.obj().height() * factor;
                unsafe {
                    let mut fbo = -1;
                    gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fbo);
                    ctx.render::<GLContext>(fbo, width, height, true).unwrap();
                }
            }
            glib::Propagation::Stop
        }
    }
}

glib::wrapper! {
    pub struct MPVGLArea(ObjectSubclass<imp::MPVGLArea>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget ,gtk::GLArea,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Default for MPVGLArea {
    fn default() -> Self {
        Self::new()
    }
}

impl MPVGLArea {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn play(&self, url: &str, name: Option<&str>, back: Option<Back>, percentage: f64) {
        let mpv = &self.imp().mpv;

        mpv.event_thread_alive
            .store(ACTIVE, std::sync::atomic::Ordering::SeqCst);
        atomic_wait::wake_all(&*mpv.event_thread_alive);

        let url = EMBY_CLIENT.get_streaming_url(url);
        mpv.load_video(&url);
        mpv.pause(true);

        mpv.set_start(percentage);

        mpv.pause(false);
    }

    pub fn add_sub(&self, url: &str) {
        self.imp().mpv.add_sub(url)
    }

    pub fn get_audio_and_subtitle_tracks(&self, count: i64) -> (Vec<MpvTrack>, Vec<MpvTrack>) {
        self.imp().mpv.get_audio_and_subtitle_tracks(count)
    }

    pub fn set_position(&self, value: f64) {
        self.imp().mpv.set_position(value)
    }

    pub fn position(&self) -> f64 {
        self.imp().mpv.position()
    }

    pub fn set_aid(&self, value: TrackSelection) {
        self.imp().mpv.set_aid(value)
    }

    pub fn set_sid(&self, value: TrackSelection) {
        self.imp().mpv.set_sid(value)
    }

    pub fn set_speed(&self, value: f64) {
        self.imp().mpv.set_speed(value)
    }

    pub fn set_volume(&self, value: i64) {
        self.imp().mpv.set_volume(value)
    }

    pub fn display_stats_toggle(&self) {
        self.imp().mpv.display_stats_toggle()
    }

    pub fn paused(&self) -> bool {
        self.imp().mpv.paused()
    }

    pub fn pause(&self, pause: bool) {
        self.imp().mpv.pause(pause)
    }
}
