use crate::f;
use serde::Serialize;
use std::{env, str::FromStr};

pub fn get_env(name: &'static str) -> core::result::Result<String, String> {
    env::var(name).map_err(|_| f!("{} not found in environment", name))
}

pub fn get_env_parse<T: FromStr>(name: &'static str) -> core::result::Result<T, String> {
    let msg = f!("Failed to parse {} into T", name);
    get_env(name).and_then(|value| value.parse::<T>().map_err(|_| msg))
}

#[macro_export]
macro_rules! lazy_lock {
    ($definition:expr) => {
        std::sync::LazyLock::new(|| $definition)
    };
    (() => $block:block) => {
        std::sync::LazyLock::new(|| $block)
    };
}

#[doc = "Return the error provided if the predicate is false"]
#[macro_export]
macro_rules! ensure {
    ($pred:expr,  $err:expr) => {
        if !$pred {
            return Err($err);
        }
    };
}

#[doc = "Return error always, this function short circuit"]
#[macro_export]
macro_rules! err {
    ($err:expr) => {
        return Err($err)
    };
}

#[macro_export]
macro_rules! lock {
    ($lock:expr) => {
        $lock.lock().unwrap()
    };
    ($lock:expr, $error:expr) => {{
        match $lock.lock() {
            | Ok(lock) => lock,
            | Err(_) => return $error,
        }
    }};
}

#[macro_export]
macro_rules! clone {
    ($expr:expr) => {
        $expr.clone()
    };
}

#[macro_export]
macro_rules! duration_since {
    ($earlier:expr) => {{
        std::time::Instant::now().duration_since($earlier)
    }};
}

#[macro_export]
macro_rules! f {
    ($($arg:tt)*) => {
        format!($($arg)*)
    };
}

#[macro_export]
macro_rules! __impl_error_display {
    ($ident:ident) => {
        impl std::error::Error for $ident {}

        impl std::fmt::Display for $ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Error: {:?}", self)
            }
        }
    };
}

#[macro_export]
macro_rules! opt {
    ($( $value:expr )?) => {{
        match ($(Some($value))?) {$(| Some(_) => Some($value),)?| _ => None}
    }};
}

#[macro_export]
macro_rules! arc {
    ($value:expr) => {
        std::sync::Arc::new($value)
    };
}

#[macro_export]
macro_rules! mutex {
    ($value:expr) => {
        std::sync::Mutex::new($value)
    };
}

#[doc = r#"
```not_rust
string!() => Empty String
string!(content) => String with content
string!(u8: content) => String from u8
string!(u8l: content) => String from lossy U8 can fail
string!(u16: content) => String from u16
string!(u16l: content) => String from lossy u16 can fail
```
"#]
#[macro_export]
macro_rules! string {
    () => {
        String::new()
    };

    ($content:expr) => {
        String::from($content)
    };

    ($content:expr, $cap:expr) => {{
        let mut string = String::with_capacity($cap);
        string.push_str($content);
        string
    }};

    // From Methods
    (u8: $content:expr) => {
        String::from_utf8($content)
    };

    (u8l: $content:expr) => {
        String::from_utf8_lossy($content)
    };

    (u16: $content:expr) => {
        String::from_utf16($content)
    };

    (u16l: $content:expr) => {
        String::from_utf16_lossy($content)
    };
}
