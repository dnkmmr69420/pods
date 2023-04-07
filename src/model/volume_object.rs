use std::cell::Cell;

use glib::prelude::*;
use glib::subclass::prelude::*;
use glib::subclass::Signal;
use glib::Properties;
use gtk::glib;
use once_cell::sync::Lazy as SyncLazy;
use once_cell::unsync::OnceCell as UnsyncOnceCell;

use crate::model;
use crate::monad_boxed_type;
use crate::podman;

monad_boxed_type!(pub(crate) BoxedVolume(podman::models::Volume) impls Debug);

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::VolumeObject)]
    pub(crate) struct VolumeObject {
        #[property(get, set, construct_only, nullable)]
        pub(super) volume_list: glib::WeakRef<model::VolumeList>,
        #[property(get, set, construct_only)]
        pub(super) volume: UnsyncOnceCell<BoxedVolume>,
        #[property(get = Self::container_list)]
        pub(super) container_list: UnsyncOnceCell<model::SimpleContainerList>,
        #[property(get)]
        pub(super) to_be_deleted: Cell<bool>,
        #[property(get, set)]
        pub(super) selected: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VolumeObject {
        const NAME: &'static str = "VolumeObject";
        type Type = super::VolumeObject;
        type Interfaces = (model::Selectable,);
    }

    impl ObjectImpl for VolumeObject {
        fn signals() -> &'static [Signal] {
            static SIGNALS: SyncLazy<Vec<Signal>> =
                SyncLazy::new(|| vec![Signal::builder("deleted").build()]);
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }

    impl VolumeObject {
        pub(super) fn container_list(&self) -> model::SimpleContainerList {
            self.container_list.get_or_init(Default::default).to_owned()
        }
    }
}

glib::wrapper! {
    pub(crate) struct VolumeObject(ObjectSubclass<imp::VolumeObject>) @implements model::Selectable;
}

impl VolumeObject {
    pub(crate) fn new(volume_list: &model::VolumeList, volume: podman::models::Volume) -> Self {
        glib::Object::builder()
            .property("volume-list", volume_list)
            .property("volume", BoxedVolume::from(volume))
            .build()
    }

    pub(super) fn emit_deleted(&self) {
        self.emit_by_name::<()>("deleted", &[]);
    }

    pub(crate) fn connect_deleted<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_local("deleted", true, move |values| {
            f(&values[0].get::<Self>().unwrap());

            None
        })
    }
}
