use std::borrow::Borrow;
use std::cell::RefCell;

use gio::prelude::*;
use gio::subclass::prelude::*;
use gtk::gio;
use gtk::glib;
use indexmap::map::IndexMap;
use once_cell::sync::Lazy as SyncLazy;

use super::AbstractVolumeListExt;
use crate::model;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct SimpleVolumeList(
        pub(super) RefCell<IndexMap<String, glib::WeakRef<model::VolumeObject>>>,
    );

    #[glib::object_subclass]
    impl ObjectSubclass for SimpleVolumeList {
        const NAME: &'static str = "SimpleVolumeList";
        type Type = super::SimpleVolumeList;
        type Interfaces = (gio::ListModel, model::AbstractVolumeList);
    }

    impl ObjectImpl for SimpleVolumeList {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: SyncLazy<Vec<glib::ParamSpec>> = SyncLazy::new(|| {
                vec![
                    glib::ParamSpecUInt::builder("len").read_only().build(),
                    glib::ParamSpecUInt::builder("used").read_only().build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = &*self.obj();
            match pspec.name() {
                "len" => obj.len().to_value(),
                "used" => obj.used().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
            model::AbstractVolumeList::bootstrap(self.obj().upcast_ref());
        }
    }

    impl ListModelImpl for SimpleVolumeList {
        fn item_type(&self) -> glib::Type {
            model::VolumeObject::static_type()
        }

        fn n_items(&self) -> u32 {
            self.0.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.0
                .borrow()
                .get_index(position as usize)
                .and_then(|(_, obj)| obj.upgrade().map(|c| c.upcast()))
        }
    }
}

glib::wrapper! {
    pub(crate) struct SimpleVolumeList(ObjectSubclass<imp::SimpleVolumeList>)
        @implements gio::ListModel, model::AbstractVolumeList;
}

impl Default for SimpleVolumeList {
    fn default() -> Self {
        glib::Object::builder().build()
    }
}

impl SimpleVolumeList {
    pub(crate) fn get(&self, index: usize) -> Option<model::VolumeObject> {
        self.imp()
            .0
            .borrow()
            .get_index(index)
            .map(|(_, c)| c)
            .and_then(glib::WeakRef::upgrade)
    }

    pub(crate) fn add_volume(&self, volume: &model::VolumeObject) {
        let (index, _) = self
            .imp()
            .0
            .borrow_mut()
            .insert_full(volume.volume().name.clone(), {
                let weak_ref = glib::WeakRef::new();
                weak_ref.set(Some(volume));
                weak_ref
            });

        self.items_changed(index as u32, 0, 1);
        self.volume_added(volume);
    }

    pub(crate) fn remove_volume<Q: Borrow<str> + ?Sized>(&self, name: &Q) {
        let mut list = self.imp().0.borrow_mut();
        if let Some((idx, _, volume)) = list.shift_remove_full(name.borrow()) {
            drop(list);
            self.items_changed(idx as u32, 1, 0);
            if let Some(volume) = volume.upgrade() {
                self.volume_removed(&volume);
            }
        }
    }

    pub(crate) fn len(&self) -> u32 {
        self.n_items()
    }

    pub(crate) fn used(&self) -> u32 {
        self.imp()
            .0
            .borrow()
            .values()
            .filter_map(glib::WeakRef::upgrade)
            .count() as u32
    }
}
