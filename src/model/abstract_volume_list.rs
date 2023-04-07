use gio::traits::ListModelExt;
use glib::prelude::*;
use glib::subclass::prelude::*;
use glib::subclass::Signal;
use gtk::gio;
use gtk::glib;
use once_cell::sync::Lazy as SyncLazy;

use crate::model;

mod imp {
    use super::*;

    #[derive(Copy, Clone, Debug)]
    pub(crate) struct AbstractVolumeList(glib::gobject_ffi::GTypeInterface);

    #[glib::object_interface]
    unsafe impl ObjectInterface for AbstractVolumeList {
        const NAME: &'static str = "AbstractVolumeList";
        type Prerequisites = (gio::ListModel,);

        fn signals() -> &'static [Signal] {
            static SIGNALS: SyncLazy<Vec<Signal>> = SyncLazy::new(|| {
                vec![
                    Signal::builder("volume-added")
                        .param_types([model::VolumeObject::static_type()])
                        .build(),
                    Signal::builder("volume-removed")
                        .param_types([model::VolumeObject::static_type()])
                        .build(),
                ]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: SyncLazy<Vec<glib::ParamSpec>> = SyncLazy::new(|| {
                vec![
                    glib::ParamSpecUInt::builder("len").read_only().build(),
                    glib::ParamSpecUInt::builder("used").read_only().build(),
                ]
            });
            PROPERTIES.as_ref()
        }
    }
}

glib::wrapper! {
    pub(crate) struct AbstractVolumeList(ObjectInterface<imp::AbstractVolumeList>)
        @requires gio::ListModel;
}

impl AbstractVolumeList {
    pub(super) fn bootstrap(list: &Self) {
        list.connect_items_changed(|self_, _, _, _| {
            self_.notify("len");
            self_.notify("used");
        });
    }
}

pub(crate) trait AbstractVolumeListExt: IsA<AbstractVolumeList> {
    fn volume_added(&self, volume: &model::VolumeObject) {
        self.emit_by_name::<()>("volume-added", &[volume]);
    }

    fn volume_name_changed(&self, volume: &model::VolumeObject) {
        self.emit_by_name::<()>("volume-name-changed", &[volume]);
    }

    fn volume_removed(&self, model: &model::VolumeObject) {
        self.emit_by_name::<()>("volume-removed", &[&model]);
    }

    fn connect_volume_added<F: Fn(&Self, &model::VolumeObject) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("volume-added", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let volume = values[1].get::<model::VolumeObject>().unwrap();
            f(&obj, &volume);

            None
        })
    }

    fn connect_volume_removed<F: Fn(&Self, &model::VolumeObject) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("volume-removed", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let volume = values[1].get::<model::VolumeObject>().unwrap();
            f(&obj, &volume);

            None
        })
    }
}

impl<T: IsA<AbstractVolumeList>> AbstractVolumeListExt for T {}

unsafe impl<T: ObjectSubclass> IsImplementable<T> for AbstractVolumeList {}
