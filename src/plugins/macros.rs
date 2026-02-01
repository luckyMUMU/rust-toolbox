//! Plugin system macros and utilities
//!
//! This module provides macros to reduce boilerplate code in plugin implementations.

/// Macro to implement the Plugin trait for wrapper plugin types.
///
/// This eliminates ~150 lines of duplicate code per plugin type.
///
/// # Usage
/// ```rust,ignore
/// impl_plugin_wrapper!(NativePlugin, crate::plugins::native::NativePlugin);
/// impl_plugin_wrapper!(PythonPlugin, crate::plugins::python::PythonPlugin);
/// ```
#[macro_export]
macro_rules! impl_plugin_wrapper {
    ($wrapper:ty, $inner:ty) => {
        impl crate::plugins::types::Plugin for $wrapper {
            fn info(&self) -> &crate::core::PluginInfo {
                &self.info
            }

            fn initialize(
                &mut self,
                config: crate::plugins::types::PluginConfig,
            ) -> crate::error::Result<()> {
                if let Some(ref mut inner) = self.inner {
                    inner.initialize(config.clone())?;
                    self.status = inner.status();
                    self.config = Some(config);
                }
                Ok(())
            }

            fn get_tools(&self) -> Vec<crate::tools::types::Tool> {
                if let Some(ref inner) = self.inner {
                    inner.get_tools()
                } else {
                    Vec::new()
                }
            }

            fn shutdown(&mut self) -> crate::error::Result<()> {
                if let Some(ref mut inner) = self.inner {
                    inner.shutdown()?;
                    self.status = inner.status();
                }
                Ok(())
            }

            fn is_initialized(&self) -> bool {
                if let Some(ref inner) = self.inner {
                    inner.is_initialized()
                } else {
                    matches!(
                        self.status,
                        crate::plugins::types::PluginStatus::Ready
                            | crate::plugins::types::PluginStatus::Running
                    )
                }
            }

            fn status(&self) -> crate::plugins::types::PluginStatus {
                if let Some(ref inner) = self.inner {
                    inner.status()
                } else {
                    self.status
                }
            }
        }
    };
}

/// Macro to generate plugin wrapper struct with common fields.
///
/// Eliminates struct definition boilerplate.
///
/// # Usage
/// ```rust,ignore
/// define_plugin_wrapper!(PythonPlugin, crate::plugins::python::PythonPlugin);
/// ```
#[macro_export]
macro_rules! define_plugin_wrapper {
    ($name:ident, $inner:ty) => {
        pub struct $name {
            pub info: crate::core::PluginInfo,
            pub status: crate::plugins::types::PluginStatus,
            pub config: Option<crate::plugins::types::PluginConfig>,
            inner: Option<$inner>,
        }
    };
}
