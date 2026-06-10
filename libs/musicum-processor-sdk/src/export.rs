/// Exports a processor type as a dynamically loadable cdylib.
///
/// # Usage
/// ```ignore
/// export_processor!(MyProcessor, Stream);
/// export_processor!(MyProcessor, Structural);
/// export_processor!(MyProcessor, Analyzer);
/// ```
///
/// `T` must implement the corresponding native trait and `Default`.
/// Generates `musicum_processor_descriptor` and `musicum_processor_create`
/// as `#[no_mangle] extern "C"` symbols.
#[macro_export]
macro_rules! export_processor {
    ($ty:ty, Stream) => {
        const _: () = {
            use $crate::ffi::{
                AbiStreamProcessor, AbiStreamProcessor_TO, ProcessorDescriptorFFI, ProcessorEntry,
            };
            use $crate::processor::{BaseProcessor, StreamProcessor};
            use $crate::abi_stable::{erased_types::TD_Opaque, std_types::{RStr, RSliceMut}};

            struct _FfiAdapter($ty);

            impl AbiStreamProcessor for _FfiAdapter {
                fn prepare(&mut self, ctx: $crate::processor::ProcessorContext) {
                    self.0.prepare(&ctx, &mut ::std::default::Default::default());
                }
                fn get_parameter(&self, id: RStr<'_>) -> f64 {
                    self.0.get_parameter(id.as_str())
                }
                fn set_parameter(&mut self, id: RStr<'_>, value: f64) {
                    self.0.set_parameter(id.as_str(), value);
                }
                fn requires_analysis(&self) -> bool {
                    self.0.requires_analysis()
                }
                fn process(&mut self, mut samples: RSliceMut<'_, f32>, time: f64, ctx: $crate::processor::ProcessorContext) {
                    self.0.process(samples.as_mut_slice(), time, &ctx);
                }
            }

            #[no_mangle]
            pub extern "C" fn musicum_processor_descriptor() -> &'static ProcessorDescriptorFFI {
                static DESC: ::std::sync::OnceLock<ProcessorDescriptorFFI> = ::std::sync::OnceLock::new();
                DESC.get_or_init(|| {
                    ProcessorDescriptorFFI::from(
                        <$ty as ::std::default::Default>::default().descriptor()
                    )
                })
            }

            #[no_mangle]
            pub extern "C" fn musicum_processor_create() -> ProcessorEntry {
                ProcessorEntry::Stream(
                    AbiStreamProcessor_TO::from_value(
                        _FfiAdapter(<$ty as ::std::default::Default>::default()),
                        TD_Opaque,
                    )
                )
            }
        };
    };

    ($ty:ty, Structural) => {
        const _: () = {
            use $crate::ffi::{
                AbiStructuralProcessor, AbiStructuralProcessor_TO, ProcessorDescriptorFFI, ProcessorEntry,
            };
            use $crate::processor::{BaseProcessor, StructuralProcessor};
            use $crate::abi_stable::{erased_types::TD_Opaque, std_types::RStr};

            struct _FfiAdapter($ty);

            impl AbiStructuralProcessor for _FfiAdapter {
                fn prepare(&mut self, ctx: $crate::processor::ProcessorContext) {
                    self.0.prepare(&ctx, &mut ::std::default::Default::default());
                }
                fn get_parameter(&self, id: RStr<'_>) -> f64 {
                    self.0.get_parameter(id.as_str())
                }
                fn set_parameter(&mut self, id: RStr<'_>, value: f64) {
                    self.0.set_parameter(id.as_str(), value);
                }
                fn requires_analysis(&self) -> bool {
                    self.0.requires_analysis()
                }
                fn segments(&self, duration: f64, ctx: $crate::processor::ProcessorContext)
                    -> $crate::abi_stable::std_types::RVec<$crate::processor::Segment>
                {
                    self.0.segments(duration, &ctx).into()
                }
            }

            #[no_mangle]
            pub extern "C" fn musicum_processor_descriptor() -> &'static ProcessorDescriptorFFI {
                static DESC: ::std::sync::OnceLock<ProcessorDescriptorFFI> = ::std::sync::OnceLock::new();
                DESC.get_or_init(|| {
                    ProcessorDescriptorFFI::from(
                        <$ty as ::std::default::Default>::default().descriptor()
                    )
                })
            }

            #[no_mangle]
            pub extern "C" fn musicum_processor_create() -> ProcessorEntry {
                ProcessorEntry::Structural(
                    AbiStructuralProcessor_TO::from_value(
                        _FfiAdapter(<$ty as ::std::default::Default>::default()),
                        TD_Opaque,
                    )
                )
            }
        };
    };

    ($ty:ty, Analyzer) => {
        const _: () = {
            use $crate::ffi::{
                AbiAnalyzer, AbiAnalyzer_TO, ProcessorDescriptorFFI, ProcessorEntry,
            };
            use $crate::analyzer::AudioAnalyser;
            use $crate::abi_stable::{erased_types::TD_Opaque, std_types::{RSlice, RStr}};

            struct _FfiAdapter($ty);

            impl AbiAnalyzer for _FfiAdapter {
                fn prepare(&mut self, _ctx: $crate::processor::ProcessorContext) {}
                fn analyze(&self, samples: RSlice<'_, f32>, ctx: $crate::processor::ProcessorContext) {
                    self.0.analyze(samples.as_slice(), &ctx, &mut ::std::default::Default::default());
                }
            }

            #[no_mangle]
            pub extern "C" fn musicum_processor_descriptor() -> &'static ProcessorDescriptorFFI {
                static DESC: ::std::sync::OnceLock<ProcessorDescriptorFFI> = ::std::sync::OnceLock::new();
                DESC.get_or_init(|| {
                    ProcessorDescriptorFFI::from(
                        <$ty as ::std::default::Default>::default().descriptor()
                    )
                })
            }

            #[no_mangle]
            pub extern "C" fn musicum_processor_create() -> ProcessorEntry {
                ProcessorEntry::Analyzer(
                    AbiAnalyzer_TO::from_value(
                        _FfiAdapter(<$ty as ::std::default::Default>::default()),
                        TD_Opaque,
                    )
                )
            }
        };
    };
}
