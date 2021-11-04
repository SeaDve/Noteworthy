use gtk::{glib, prelude::*};

use std::path::PathBuf;

// Taken from fractal-next GPLv3
// See https://gitlab.gnome.org/GNOME/fractal/-/blob/fractal-next/src/utils.rs
/// Spawns a future in the main context
#[macro_export]
macro_rules! spawn {
    ($future:expr) => {
        let ctx = glib::MainContext::default();
        ctx.spawn_local($future);
    };
    ($priority:expr, $future:expr) => {
        let ctx = glib::MainContext::default();
        ctx.spawn_local_with_priority($priority, $future);
    };
}

/// Pushes a function to be executed in the main thread pool
#[macro_export]
macro_rules! spawn_blocking {
    ($function:expr) => {
        crate::THREAD_POOL.push_future($function).unwrap()
    };
}

pub trait PropExpr {
    /// Create a constant expression looking up an object's property
    fn property_expression(&self, prop_name: &str) -> gtk::Expression;

    /// Create a non-constant expression looking up an object's property
    fn weak_property_expression(&self, prop_name: &str) -> gtk::Expression;
}

impl<T: IsA<glib::Object>> PropExpr for T {
    fn property_expression(&self, prop_name: &str) -> gtk::Expression {
        let obj_expr = gtk::ConstantExpression::new(self).upcast();
        gtk::PropertyExpression::new(T::static_type(), Some(&obj_expr), prop_name).upcast()
    }

    fn weak_property_expression(&self, prop_name: &str) -> gtk::Expression {
        let obj_expr = gtk::ObjectExpression::new(self).upcast();
        gtk::PropertyExpression::new(T::static_type(), Some(&obj_expr), prop_name).upcast()
    }
}

pub trait ChainExpr {
    /// Create an expression with prop_name chained from self
    fn property_expression(&self, prop_name: &str) -> gtk::Expression;

    /// Create a closure expression chained from self
    fn closure_expression<F, T>(self, f: F) -> gtk::Expression
    where
        F: Fn(&[glib::Value]) -> T + 'static,
        T: glib::value::ValueType;
}

impl ChainExpr for gtk::Expression {
    fn property_expression(&self, prop_name: &str) -> gtk::Expression {
        gtk::PropertyExpression::new(self.value_type(), Some(self), prop_name).upcast()
    }

    fn closure_expression<F, T>(self, f: F) -> gtk::Expression
    where
        F: Fn(&[glib::Value]) -> T + 'static,
        T: glib::value::ValueType,
    {
        gtk::ClosureExpression::new(f, &[self]).upcast()
    }
}

pub fn default_notes_dir() -> PathBuf {
    let mut data_dir = glib::user_data_dir();
    data_dir.push("Notes");
    data_dir
}

pub fn generate_unique_file_name(prefix: &str) -> String {
    let formatted_time = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S-%f");
    format!("{}-{}", prefix, formatted_time)
}
