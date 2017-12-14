#[macro_export]
macro_rules! add_getter {
    ($i:ident => $([$get:ident, $c_get:ident, $t:ty]),+) => {
        impl $i {
            $(
                pub fn $get(&self) -> $t {
                    self.as_ref().$c_get
                }
            )*
        }
    }
}

#[macro_export]
macro_rules! add_setter {
    ($i:ident => $([$set:ident, $c_set:ident, $t:ty]),+) => {
        impl $i {
            $(
                pub fn $set(&self, item: $t) -> () {
                    self.as_ref().$c_get = item
                }
            )*
        }
    }
}