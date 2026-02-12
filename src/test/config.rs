use crate::*;

pub static CONFIG_TEST_MUTEX: LazyLock<std::sync::Mutex<()>> =
    LazyLock::new(|| std::sync::Mutex::new(()));

#[macro_export]
macro_rules! test_config {
    ($e:expr) => {
        struct ConfigGuard<'a> {
            _guard: std::sync::MutexGuard<'a, ()>,
        }

        impl<'a> Drop for ConfigGuard<'a> {
            fn drop(&mut self) {
                config::init_config(config::Root { ..default!() }).unwrap()
            }
        }

        let _config_guard = ConfigGuard {
            _guard: test::CONFIG_TEST_MUTEX.lock()?,
        };
        config::init!($e);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test::case]
    fn test_config() {
        test::config!(config::Root {
            private_key_hex: "aaa".to_string(),
            ..default!()
        });
        assert_eq!(config!().private_key_hex, "aaa");
    }

    #[test::case]
    fn test_re_config() {
        test::config!(config::Root {
            private_key_hex: "bbb".to_string(),
            ..default!()
        });
        assert_eq!(config!().private_key_hex, "bbb");
    }
}
