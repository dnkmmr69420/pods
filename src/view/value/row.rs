use std::cell::RefCell;

use adw::subclass::prelude::*;
use gettextrs::gettext;
use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::CompositeTemplate;

use crate::model;

const ACTION_REMOVE: &str = "value-row.remove";

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::Row)]
    #[template(resource = "/com/github/marhkb/Pods/ui/value/row.ui")]
    pub(crate) struct Row {
        #[property(get, set = Self::set_value, construct)]
        pub(super) value: RefCell<Option<model::Value>>,
        pub(super) bindings: RefCell<Vec<glib::Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Row {
        const NAME: &'static str = "PdsValueRow";
        type Type = super::Row;
        type ParentType = adw::EntryRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.install_action(ACTION_REMOVE, None, |widget, _, _| {
                if let Some(value) = widget.value() {
                    value.remove_request();
                }
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
    }

    impl WidgetImpl for Row {}
    impl ListBoxRowImpl for Row {}
    impl PreferencesRowImpl for Row {}
    impl EntryRowImpl for Row {}

    impl Row {
        pub(super) fn set_value(&self, value: Option<model::Value>) {
            let obj = &*self.obj();
            if obj.value() == value {
                return;
            }

            let mut bindings = self.bindings.borrow_mut();

            while let Some(binding) = bindings.pop() {
                binding.unbind();
            }

            if let Some(ref value) = value {
                let binding = value
                    .bind_property("value", obj, "text")
                    .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                    .build();
                bindings.push(binding);
            }

            self.value.replace(value);
            obj.notify("value");
        }
    }
}

glib::wrapper! {
    pub(crate) struct Row(ObjectSubclass<imp::Row>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::EntryRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget, gtk::Editable;
}

impl From<&model::Value> for Row {
    fn from(value: &model::Value) -> Self {
        Self::new(value, &gettext("Value"))
    }
}

impl Row {
    pub fn new(value: &model::Value, title: &str) -> Self {
        glib::Object::builder()
            .property("value", value)
            .property("title", title)
            .build()
    }
}
