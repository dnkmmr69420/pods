use std::cell::RefCell;

use glib::clone;
use glib::closure;
use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::model;
use crate::model::SelectableExt;
use crate::model::SelectableListExt;
use crate::utils;
use crate::view;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::Row)]
    #[template(resource = "/com/github/marhkb/Pods/ui/volume2/row.ui")]
    pub(crate) struct Row {
        pub(super) bindings: RefCell<Vec<glib::Binding>>,
        #[property(get, set = Self::set_volume, construct, explicit_notify, nullable)]
        pub(super) volume: glib::WeakRef<model::VolumeObject>,
        #[template_child]
        pub(super) check_button_revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub(super) check_button: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub(super) name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) end_box_revealer: TemplateChild<gtk::Revealer>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Row {
        const NAME: &'static str = "PdsVolumeRow2";
        type Type = super::Row;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action("volume-row.activate", None, |widget, _, _| {
                widget.activate();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Row {
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

            let volume_expr = Self::Type::this_expression("volume");

            let selection_mode_expr = volume_expr
                .chain_property::<model::VolumeObject>("volume-list")
                .chain_property::<model::VolumeList>("selection-mode");

            selection_mode_expr.bind(&*self.check_button_revealer, "reveal-child", Some(obj));
            selection_mode_expr
                .chain_closure::<bool>(closure!(|_: Self::Type, is_selection_mode: bool| {
                    !is_selection_mode
                }))
                .bind(&*self.end_box_revealer, "reveal-child", Some(obj));

            gtk::ClosureExpression::new::<String>(
                [
                    volume_expr
                        .chain_property::<model::VolumeObject>("volume")
                        .chain_closure::<String>(closure!(
                            |_: Self::Type, volume: model::BoxedVolume| {
                                utils::format_id(&volume.name)
                            }
                        ))
                        .upcast_ref(),
                    volume_expr
                        .chain_property::<model::VolumeObject>("to-be-deleted")
                        .upcast_ref(),
                ],
                closure!(|_: Self::Type, name: String, to_be_deleted: bool| {
                    if to_be_deleted {
                        format!("<s>{name}</s>")
                    } else {
                        name
                    }
                }),
            )
            .bind(&*self.name_label, "label", Some(obj));

            // let css_classes = utils::css_classes(self.name_label.upcast_ref());
            // image_expr
            //     .chain_property::<model::Image>("repo-tags")
            //     .chain_property::<model::RepoTagList>("len")
            //     .chain_closure::<Vec<String>>(closure!(|_: Self::Type, len: u32| {
            //         css_classes
            //             .iter()
            //             .cloned()
            //             .chain(if len == 0 {
            //                 Some(String::from("dim-label"))
            //             } else {
            //                 None
            //             })
            //             .collect::<Vec<_>>()
            //     }))
            //     .bind(&*self.name_label, "css-classes", Some(obj));

            if let Some(volume) = obj.volume() {
                obj.action_set_enabled("volume.show-details", !volume.to_be_deleted());
                volume.connect_notify_local(
                    Some("to-be-deleted"),
                    clone!(@weak obj => move|volume, _| {
                        obj.action_set_enabled("volume.show-details", !volume.to_be_deleted());
                    }),
                );
            }
        }
    }

    impl WidgetImpl for Row {}
    impl ListBoxRowImpl for Row {}

    impl Row {
        pub(super) fn set_volume(&self, value: Option<&model::VolumeObject>) {
            let obj = &*self.obj();
            if obj.volume().as_ref() == value {
                return;
            }

            let mut bindings = self.bindings.borrow_mut();
            while let Some(binding) = bindings.pop() {
                binding.unbind();
            }

            if let Some(volume) = value {
                let binding = volume
                    .bind_property("selected", &*self.check_button, "active")
                    .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                    .build();

                bindings.push(binding);
            }

            self.volume.set(value);
            obj.notify("volume")
        }
    }
}

glib::wrapper! {
    pub(crate) struct Row(ObjectSubclass<imp::Row>)
        @extends gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Accessible, gtk::Buildable, gtk::Actionable, gtk::ConstraintTarget;

}

impl From<&model::VolumeObject> for Row {
    fn from(volume: &model::VolumeObject) -> Self {
        glib::Object::builder().property("volume", volume).build()
    }
}

impl Row {
    pub(crate) fn activate(&self) {
        if let Some(volume) = self.volume().as_ref() {
            if volume
                .volume_list()
                .map(|list| list.is_selection_mode())
                .unwrap_or(false)
            {
                volume.select();
            } else {
                utils::find_leaflet_overlay(self.upcast_ref())
                    .show_details(view::VolumeDetailsPage::from(volume).upcast_ref());
            }
        }
    }
}
