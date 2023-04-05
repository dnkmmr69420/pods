use std::cell::RefCell;

use adw::traits::BinExt;
use gettextrs::gettext;
use glib::clone;
use glib::closure;
use glib::Properties;
use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::model;
use crate::utils;
use crate::view;

const ACTION_INSPECT_IMAGE: &str = "image-details-page.inspect-image";
const ACTION_DELETE_IMAGE: &str = "image-details-page.delete-image";

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::DetailsPage)]
    #[template(resource = "/com/github/marhkb/Pods/ui/volume2/details-page.ui")]
    pub(crate) struct DetailsPage {
        pub(super) handler_id: RefCell<Option<glib::SignalHandlerId>>,
        #[property(get, set = Self::set_volume, construct, explicit_notify, nullable)]
        pub(super) volume: glib::WeakRef<model::VolumeObject>,
        #[template_child]
        pub(super) back_navigation_controls: TemplateChild<view::BackNavigationControls>,
        #[template_child]
        pub(super) window_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub(super) name_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) created_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) size_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) command_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) entrypoint_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) ports_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) leaflet_overlay: TemplateChild<view::LeafletOverlay>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DetailsPage {
        const NAME: &'static str = "PdsVolumeDetailsPage";
        type Type = super::DetailsPage;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action(ACTION_INSPECT_IMAGE, None, |widget, _, _| {
                widget.show_inspection();
            });

            klass.install_action(ACTION_DELETE_IMAGE, None, |widget, _, _| {
                widget.delete_volume();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DetailsPage {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = &*self.obj();

            // let volume_expr = Self::Type::this_expression("volume");
            // let data_expr = volume_expr.chain_property::<model::Image>("data");
            // let image_config_expr = data_expr.chain_property::<model::ImageData>("config");
            // let cmd_expr = image_config_expr.chain_property::<model::ImageConfig>("cmd");
            // let entrypoint_expr =
            //     image_config_expr.chain_property::<model::ImageConfig>("entrypoint");
            // let exposed_ports_expr =
            //     image_config_expr.chain_property::<model::ImageConfig>("exposed-ports");

            // volume_expr
            //     .chain_property::<model::Image>("to-be-deleted")
            //     .watch(
            //         Some(obj),
            //         clone!(@weak obj => move || {
            //             obj.action_set_enabled(
            //                 ACTION_DELETE_IMAGE,
            //                 obj.volume().map(|volume| !volume.to_be_deleted()).unwrap_or(false),
            //             );
            //         }),
            //     );

            // data_expr
            //     .chain_closure::<bool>(closure!(|_: Self::Type, cmd: Option<model::ImageData>| {
            //         cmd.is_none()
            //     }))
            //     .bind(&*self.inspection_spinner, "visible", Some(obj));

            // volume_expr
            //     .chain_property::<model::Image>("id")
            //     .chain_closure::<String>(closure!(|_: Self::Type, id: &str| utils::format_id(id)))
            //     .bind(&*self.id_row, "value", Some(obj));

            // gtk::ClosureExpression::new::<String>(
            //     &[
            //         Self::Type::this_expression("root")
            //             .chain_property::<gtk::Window>("application")
            //             .chain_property::<crate::Application>("ticks"),
            //         volume_expr.chain_property::<model::Image>("created"),
            //     ],
            //     closure!(|_: Self::Type, _ticks: u64, created: i64| {
            //         utils::format_ago(utils::timespan_now(created))
            //     }),
            // )
            // .bind(&*self.created_row, "value", Some(obj));

            // gtk::ClosureExpression::new::<String>(
            //     &[
            //         volume_expr.chain_property::<model::Image>("size").upcast(),
            //         volume_expr
            //             .chain_property::<model::Image>("shared-size")
            //             .upcast(),
            //         volume_expr
            //             .chain_property::<model::Image>("virtual-size")
            //             .upcast(),
            //     ],
            //     closure!(
            //         |_: Self::Type, size: u64, shared_size: u64, virtual_size: u64| {
            //             let formatted_size = glib::format_size(size);
            //             if size == shared_size {
            //                 if shared_size == virtual_size {
            //                     formatted_size.to_string()
            //                 } else {
            //                     gettext!(
            //                         // Translators: "{}" are placeholders for storage space.
            //                         "{} (Virtual: {})",
            //                         formatted_size,
            //                         glib::format_size(virtual_size),
            //                     )
            //                 }
            //             } else if size == virtual_size {
            //                 if shared_size > 0 {
            //                     gettext!(
            //                         // Translators: "{}" are placeholders for storage space.
            //                         "{} (Shared: {})",
            //                         formatted_size,
            //                         glib::format_size(shared_size),
            //                     )
            //                 } else {
            //                     formatted_size.to_string()
            //                 }
            //             } else {
            //                 gettext!(
            //                     // Translators: "{}" are placeholders for storage space.
            //                     "{} (Shared: {}, Virtual: {})",
            //                     formatted_size,
            //                     glib::format_size(shared_size),
            //                     glib::format_size(virtual_size),
            //                 )
            //             }
            //         }
            //     ),
            // )
            // .bind(&*self.size_row, "value", Some(obj));

            // cmd_expr.bind(&*self.command_row, "value", Some(obj));
            // cmd_expr
            //     .chain_closure::<bool>(closure!(|_: Self::Type, cmd: Option<&str>| {
            //         cmd.is_some()
            //     }))
            //     .bind(&*self.command_row, "visible", Some(obj));

            // entrypoint_expr.bind(&*self.entrypoint_row, "value", Some(obj));
            // entrypoint_expr
            //     .chain_closure::<bool>(closure!(|_: Self::Type, entrypoint: Option<&str>| {
            //         entrypoint.is_some()
            //     }))
            //     .bind(&*self.entrypoint_row, "visible", Some(obj));

            // exposed_ports_expr
            //     .chain_closure::<String>(closure!(
            //         |_: Self::Type, exposed_ports: gtk::StringList| {
            //             let exposed_ports = exposed_ports
            //                 .iter::<glib::Object>()
            //                 .map(|obj| {
            //                     obj.unwrap()
            //                         .downcast::<gtk::StringObject>()
            //                         .unwrap()
            //                         .string()
            //                 })
            //                 .collect::<Vec<_>>();

            //             utils::format_iter(exposed_ports.iter().map(glib::GString::as_str), ", ")
            //         }
            //     ))
            //     .bind(&*self.ports_row, "value", Some(obj));

            // exposed_ports_expr
            //     .chain_closure::<bool>(closure!(|_: Self::Type, exposed_ports: gtk::StringList| {
            //         exposed_ports.n_items() > 0
            //     }))
            //     .bind(&*self.ports_row, "visible", Some(obj));
        }

        fn dispose(&self) {
            utils::ChildIter::from(self.obj().upcast_ref()).for_each(|child| child.unparent());
        }
    }

    impl WidgetImpl for DetailsPage {}

    impl DetailsPage {
        pub(super) fn set_volume(&self, value: Option<&model::VolumeObject>) {
            let obj = &*self.obj();
            if obj.volume().as_ref() == value {
                return;
            }

            self.window_title.set_subtitle("");
            if let Some(volume) = obj.volume() {
                volume.disconnect(self.handler_id.take().unwrap());
            }

            if let Some(volume) = value {
                self.window_title
                    .set_subtitle(&utils::format_id(&volume.volume().name));
                // image.inspect(clone!(@weak obj => move |e| {
                //     utils::show_error_toast(obj.upcast_ref(), &gettext("Error on loading volume details"), &e.to_string());
                // }));

                let handler_id = volume.connect_deleted(clone!(@weak obj => move |volume| {
                    utils::show_toast(obj.upcast_ref(), gettext!("Volume '{}' has been deleted", volume.volume().name));
                    obj.imp().back_navigation_controls.navigate_back();
                }));
                self.handler_id.replace(Some(handler_id));
            }

            self.volume.set(value);
            obj.notify("volume");
        }
    }
}

glib::wrapper! {
    pub(crate) struct DetailsPage(ObjectSubclass<imp::DetailsPage>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl From<&model::VolumeObject> for DetailsPage {
    fn from(volume: &model::VolumeObject) -> Self {
        glib::Object::builder().property("volume", volume).build()
    }
}

impl DetailsPage {
    fn show_inspection(&self) {
        self.exec_action(|| {
            if let Some(volume) = self.volume() {
                let weak_ref = glib::WeakRef::new();
                weak_ref.set(Some(&volume));

                self.imp().leaflet_overlay.show_details(
                    view::ScalableTextViewPage::from(view::Entity::Volume(weak_ref)).upcast_ref(),
                );
            }
        });
    }

    fn delete_volume(&self) {
        self.exec_action(|| {
            super::delete_image_show_confirmation(self.upcast_ref(), self.volume());
        });
    }

    fn exec_action<F: Fn()>(&self, op: F) {
        if self.imp().leaflet_overlay.child().is_none() {
            op();
        }
    }
}
