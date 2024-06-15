pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[macro_export]
macro_rules! ensure {
    ( $cond:expr, $arg:tt ) => {
        if !($cond) {
            return Err(($arg))?
        }
    };
}

#[macro_export]
macro_rules! bail {
    ( $arg:literal ) => {
        return Err(($arg))?
    };
    ( $arg:expr ) => {
        return Err(($arg))?
    };
}

pub trait Context<T> {
    fn context(self, message: &'static str) -> Result<T>;
}

impl<T> Context<T> for Option<T> {
    fn context(self, message: &'static str) -> Result<T> {
        match self {
            Some(e) => Ok(e),
            None => Err(message)?
        }
    }
}

pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    std::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}
