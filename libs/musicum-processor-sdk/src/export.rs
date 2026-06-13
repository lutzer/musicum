/// Exports a processor type as a dynamically loadable cdylib.
///
/// `T` must implement `BaseProcessor`, `ProcessorMeta`, and `Default`.
/// The `with:` form additionally emits analyzer symbols via `export_analyzer!`.
#[macro_export]
macro_rules! export_processor {
    ($ty:ty) => {
        const _: () = {
            use $crate::ffi::{AbiProcessor_TO, FfiAdapter, ProcessorDescriptorFFI};
            use $crate::processor::ProcessorMeta;
            use $crate::abi_stable::{erased_types::TD_Opaque, std_types::RBox};

            #[no_mangle]
            pub extern "C" fn musicum_processor_descriptor() -> &'static ProcessorDescriptorFFI {
                static DESC: ::std::sync::OnceLock<ProcessorDescriptorFFI> =
                    ::std::sync::OnceLock::new();
                DESC.get_or_init(|| ProcessorDescriptorFFI::from(<$ty>::descriptor()))
            }

            #[no_mangle]
            pub extern "C" fn musicum_processor_create() -> AbiProcessor_TO<'static, RBox<()>> {
                AbiProcessor_TO::from_value(
                    FfiAdapter(<$ty as ::std::default::Default>::default()),
                    TD_Opaque,
                )
            }
        };
    };
    ($ty:ty, with: $an:ty = $id:expr) => {
        $crate::export_processor!($ty);
        $crate::export_analyzer!($an, $id);
    };
}

/// Exports an audio analyzer as a dynamically loadable cdylib.
/// `T` must implement `AudioAnalyser` and `Default`.
#[macro_export]
macro_rules! export_analyzer {
    ($an:ty, $id:expr) => {
        const _: () = {
            use $crate::ffi::{AbiAnalyzer_TO, FfiAnalyzerAdapter};
            use $crate::abi_stable::{
                erased_types::TD_Opaque,
                std_types::{RBox, RStr},
            };

            #[no_mangle]
            pub extern "C" fn musicum_analyzer_create() -> AbiAnalyzer_TO<'static, RBox<()>> {
                AbiAnalyzer_TO::from_value(
                    FfiAnalyzerAdapter(<$an as ::std::default::Default>::default()),
                    TD_Opaque,
                )
            }
        };
    };
}
