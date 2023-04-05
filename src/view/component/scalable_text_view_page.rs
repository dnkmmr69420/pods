use ashpd::desktop::file_chooser::SaveFileRequest;
use ashpd::WindowIdentifier;
use gettextrs::gettext;
use gtk::gdk;
use gtk::gio;
use gtk::glib;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use once_cell::unsync::OnceCell;
use sourceview5::traits::BufferExt;

use crate::model;
use crate::utils;
use crate::view;

const ACTION_SAVE_TO_FILE: &str = "source-view-page.save-to-file";
const ACTION_ENTER_SEARCH: &str = "source-view-page.enter-search";
const ACTION_EXIT_SEARCH: &str = "source-view-page.exit-search";
const ACTION_ZOOM_OUT: &str = "source-view-page.zoom-out";
const ACTION_ZOOM_IN: &str = "source-view-page.zoom-in";
const ACTION_ZOOM_NORMAL: &str = "source-view-page.zoom-normal";

#[derive(Clone, Debug)]
pub(crate) enum Entity {
    Image(glib::WeakRef<model::Image>),
    Container {
        container: glib::WeakRef<model::Container>,
        mode: Mode,
    },
    Pod {
        pod: glib::WeakRef<model::Pod>,
        mode: Mode,
    },
    Volume(glib::WeakRef<model::VolumeObject>),
}
impl Entity {
    fn filename(&self) -> String {
        match self {
            Self::Image(image) => format!("{}.json", image.upgrade().unwrap().id()),
            Self::Container { container, mode } => {
                format!(
                    "{}.{}",
                    container.upgrade().unwrap().name(),
                    mode.file_ext()
                )
            }
            Self::Pod { pod, mode } => {
                format!("{}.{}", pod.upgrade().unwrap().name(), mode.file_ext())
            }
            Self::Volume(volume) => {
                format!("{}.json", volume.upgrade().unwrap().volume().name)
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Mode {
    Inspect,
    Kube,
}
impl Mode {
    fn file_ext(&self) -> &str {
        match self {
            Self::Inspect => "json",
            Self::Kube => "yaml",
        }
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/marhkb/Pods/ui/component/scalable-text-view-page.ui")]
    pub(crate) struct ScalableTextViewPage {
        pub(super) entity: OnceCell<Entity>,
        #[template_child]
        pub(super) zoom_control: TemplateChild<view::ZoomControl>,
        #[template_child]
        pub(super) window_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub(super) menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub(super) search_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub(super) search_bar: TemplateChild<gtk::SearchBar>,
        #[template_child]
        pub(super) search_widget: TemplateChild<view::SourceViewSearchWidget>,
        #[template_child]
        pub(super) stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub(super) source_view: TemplateChild<view::ScalableTextView>,
        #[template_child]
        pub(super) source_buffer: TemplateChild<sourceview5::Buffer>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScalableTextViewPage {
        const NAME: &'static str = "PdsScalableTextViewPage";
        type Type = super::ScalableTextViewPage;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.install_action_async(ACTION_SAVE_TO_FILE, None, |widget, _, _| async move {
                widget.save_to_file().await;
            });

            klass.install_action(ACTION_ENTER_SEARCH, None, |widget, _, _| {
                widget.enter_search();
            });
            klass.add_binding_action(
                gdk::Key::F,
                gdk::ModifierType::CONTROL_MASK,
                ACTION_ENTER_SEARCH,
                None,
            );

            klass.install_action(ACTION_EXIT_SEARCH, None, |widget, _, _| {
                widget.exit_search();
            });

            klass.add_binding_action(
                gdk::Key::Escape,
                gdk::ModifierType::empty(),
                ACTION_EXIT_SEARCH,
                None,
            );

            klass.install_action(ACTION_ZOOM_OUT, None, |widget, _, _| {
                widget.imp().source_view.zoom_out();
            });
            klass.install_action(ACTION_ZOOM_IN, None, |widget, _, _| {
                widget.imp().source_view.zoom_in();
            });
            klass.install_action(ACTION_ZOOM_NORMAL, None, |widget, _, _| {
                widget.imp().source_view.zoom_normal();
            });

            klass.add_binding_action(
                gdk::Key::minus,
                gdk::ModifierType::CONTROL_MASK,
                ACTION_ZOOM_OUT,
                None,
            );
            klass.add_binding_action(
                gdk::Key::KP_Subtract,
                gdk::ModifierType::CONTROL_MASK,
                ACTION_ZOOM_OUT,
                None,
            );

            klass.add_binding_action(
                gdk::Key::plus,
                gdk::ModifierType::CONTROL_MASK,
                ACTION_ZOOM_IN,
                None,
            );
            klass.add_binding_action(
                gdk::Key::KP_Add,
                gdk::ModifierType::CONTROL_MASK,
                ACTION_ZOOM_IN,
                None,
            );
            klass.add_binding_action(
                gdk::Key::equal,
                gdk::ModifierType::CONTROL_MASK,
                ACTION_ZOOM_IN,
                None,
            );

            klass.add_binding_action(
                gdk::Key::_0,
                gdk::ModifierType::CONTROL_MASK,
                ACTION_ZOOM_NORMAL,
                None,
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl ScalableTextViewPage {
        #[template_callback]
        fn on_scroll(&self, _dx: f64, dy: f64, scroll: gtk::EventControllerScroll) -> gtk::Inhibit {
            gtk::Inhibit(
                if scroll.current_event_state() == gdk::ModifierType::CONTROL_MASK {
                    let view = &*self.source_view;
                    if dy.is_sign_negative() {
                        view.zoom_in();
                    } else {
                        view.zoom_out();
                    }
                    true
                } else {
                    false
                },
            )
        }
    }

    impl ObjectImpl for ScalableTextViewPage {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = &*self.obj();

            self.menu_button
                .popover()
                .unwrap()
                .downcast::<gtk::PopoverMenu>()
                .unwrap()
                .add_child(&*self.zoom_control, "zoom-control");

            self.search_bar.connect_search_mode_enabled_notify(
                clone!(@weak obj => move |search_bar| {
                    let search_entry = &*obj.imp().search_widget;
                    if search_bar.is_search_mode() {
                        search_entry.grab_focus();
                    } else {
                        search_entry.set_text("");
                    }
                }),
            );

            self.search_button
                .bind_property("active", &*self.search_bar, "search-mode-enabled")
                .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                .build();

            self.search_widget
                .set_source_view(Some(self.source_view.upcast_ref()));

            let adw_style_manager = adw::StyleManager::default();
            obj.on_notify_dark(&adw_style_manager);
            adw_style_manager.connect_dark_notify(clone!(@weak obj => move |style_manager| {
                obj.on_notify_dark(style_manager);
            }));
        }

        fn dispose(&self) {
            utils::ChildIter::from(self.obj().upcast_ref()).for_each(|child| child.unparent());
        }
    }

    impl WidgetImpl for ScalableTextViewPage {}
}

glib::wrapper! {
    pub(crate) struct ScalableTextViewPage(ObjectSubclass<imp::ScalableTextViewPage>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl From<Entity> for ScalableTextViewPage {
    fn from(entity: Entity) -> Self {
        let obj: Self = glib::Object::builder().build();
        let imp = obj.imp();

        match &entity {
            Entity::Image(image) => {
                imp.window_title.set_title(&gettext("Image Inspection"));
                if let Some(image) = image.upgrade() {
                    imp.window_title
                        .set_subtitle(&utils::format_id(&image.id()));
                }
            }
            Entity::Container { mode, container } => {
                imp.window_title.set_title(&match mode {
                    Mode::Inspect => gettext("Container Inspection"),
                    Mode::Kube => gettext("Container Kube Generation"),
                });
                model::Container::this_expression("name").bind(
                    &*imp.window_title,
                    "subtitle",
                    container.upgrade().as_ref(),
                );
            }
            Entity::Pod { mode, pod } => {
                imp.window_title.set_title(&match mode {
                    Mode::Inspect => gettext("Pod Inspection"),
                    Mode::Kube => gettext("Pod Kube Generation"),
                });
                if let Some(pod) = pod.upgrade() {
                    imp.window_title.set_subtitle(&pod.name());
                }
            }
            Entity::Volume(volume) => {
                imp.window_title.set_title(&gettext("Volume Inspection"));
                if let Some(volume) = volume.upgrade() {
                    imp.window_title
                        .set_subtitle(&utils::format_id(&volume.volume().name));
                }
            }
        }

        let language = match &entity {
            Entity::Image(_) | Entity::Volume(_) => "json",
            Entity::Container { mode, .. } => match mode {
                Mode::Inspect => "json",
                Mode::Kube => "yaml",
            },
            Entity::Pod { mode, .. } => match mode {
                Mode::Inspect => "json",
                Mode::Kube => "yaml",
            },
        };

        match sourceview5::LanguageManager::default().language(language) {
            Some(lang) => imp.source_buffer.set_language(Some(&lang)),
            None => {
                log::warn!("Could not set language to '{language}'");
                utils::show_toast(
                    obj.upcast_ref(),
                    gettext!("Could not set language to '{}'", language),
                );
            }
        }

        match entity.clone() {
            Entity::Image(image) => {
                let api = image.upgrade().unwrap().api().unwrap();

                utils::do_async(
                    async move {
                        api.inspect()
                            .await
                            .map_err(anyhow::Error::from)
                            .and_then(|data| {
                                serde_json::to_string_pretty(&data).map_err(anyhow::Error::from)
                            })
                    },
                    clone!(@weak obj => move |result| obj.init(result, Mode::Inspect)),
                );
            }
            Entity::Container { container, mode } => {
                let api = container.upgrade().unwrap().api().unwrap();

                utils::do_async(
                    async move {
                        match mode {
                            Mode::Inspect => api
                                .inspect()
                                .await
                                .map_err(anyhow::Error::from)
                                .and_then(|data| {
                                    serde_json::to_string_pretty(&data).map_err(anyhow::Error::from)
                                }),
                            Mode::Kube => api
                                .generate_kube_yaml(false)
                                .await
                                .map_err(anyhow::Error::from),
                        }
                    },
                    clone!(@weak obj => move |result| obj.init(result, mode)),
                );
            }
            Entity::Pod { pod, mode } => {
                let api = pod.upgrade().unwrap().api().unwrap();

                utils::do_async(
                    async move {
                        match mode {
                            Mode::Inspect => api
                                .inspect()
                                .await
                                .map_err(anyhow::Error::from)
                                .and_then(|data| {
                                    serde_json::to_string_pretty(&data).map_err(anyhow::Error::from)
                                }),
                            Mode::Kube => api
                                .generate_kube_yaml(false)
                                .await
                                .map_err(anyhow::Error::from),
                        }
                    },
                    clone!(@weak obj => move |result| obj.init(result, mode)),
                );
            }
            Entity::Volume(volume) => {
                obj.init(
                    serde_json::to_string_pretty(&*volume.upgrade().unwrap().volume())
                        .map_err(anyhow::Error::from),
                    Mode::Inspect,
                );
            }
        };

        imp.entity.set(entity).unwrap();

        obj
    }
}

impl ScalableTextViewPage {
    fn init(&self, result: anyhow::Result<String>, mode: Mode) {
        let imp = self.imp();
        match result {
            Ok(text) => {
                imp.source_buffer.set_text(&text);
                imp.stack.set_visible_child_name("loaded");
            }
            Err(e) => {
                imp.spinner.set_spinning(false);
                utils::show_error_toast(
                    self.upcast_ref(),
                    &match mode {
                        Mode::Inspect => gettext("Inspection error"),
                        Mode::Kube => gettext("Kube generation error"),
                    },
                    &e.to_string(),
                );
                utils::parent_leaflet_overlay(self.upcast_ref())
                    .unwrap()
                    .hide_details();
            }
        }
    }

    async fn save_to_file(&self) {
        let imp = self.imp();

        let request = SaveFileRequest::default()
            .identifier(WindowIdentifier::from_native(&self.native().unwrap()).await)
            .current_name(imp.entity.get().unwrap().filename().as_str())
            .modal(true);

        utils::show_save_file_dialog(
            request,
            self.upcast_ref(),
            clone!(@weak self as obj => move |files| {
                let file = gio::File::for_uri(files.uris()[0].as_str());

                if let Some(path) = file.path() {
                    let file = std::fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(path)
                        .unwrap();

                    let buffer = &*obj.imp().source_buffer;
                    let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);

                    glib::MainContext::default().spawn_local(clone!(@weak obj => async move {
                        if let Err((msg, _)) = gio::WriteOutputStream::new(file)
                            .write_all_future(text, glib::Priority::default())
                            .await
                        {
                            utils::show_error_toast(obj.upcast_ref(), &gettext("Error"), &msg);
                        }
                    }));
                }
            }),
        )
        .await;
    }

    fn enter_search(&self) {
        let imp = self.imp();
        imp.search_button.set_active(true);
        imp.search_widget.delete_text(0, -1);
        imp.search_widget.grab_focus();
    }

    fn exit_search(&self) {
        self.imp().search_button.set_active(false);
    }

    fn on_notify_dark(&self, style_manager: &adw::StyleManager) {
        self.imp().source_buffer.set_style_scheme(
            sourceview5::StyleSchemeManager::default()
                .scheme(if style_manager.is_dark() {
                    "Adwaita-dark"
                } else {
                    "Adwaita"
                })
                .as_ref(),
        );
    }
}
