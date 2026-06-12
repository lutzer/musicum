/// Exports a processor type as a dynamically loadable cdylib.
///
/// # Usage
/// ```ignore
/// export_processor!(MyProcessor, Stream);
/// export_processor!(MyProcessor, Structural);
/// export_processor!(MyProcessor, Analyzer);
/// export_processor!(MyProcessor, Stream, with: MyAnalyzer = "my_analyzer");
/// export_processor!(MyProcessor, Structural, with: MyAnalyzer = "my_analyzer");
/// ```
///
/// `T` must implement the corresponding native trait and `Default`.
/// Generates `musicum_processor_descriptor` and `musicum_processor_create`
/// as `#[no_mangle] extern "C"` symbols. The `with:` form additionally
/// emits `musicum_analyzer_descriptor` and `musicum_analyzer_create`.
#[macro_export]
macro_rules! export_processor {
    ($ty:ty, Stream) => {
        const _: () = {
            use $crate::ffi::{
                AbiStreamProcessor, AbiStreamProcessor_TO,
                AnalysisContextFFI, ProcessorDescriptorFFI, ProcessorEntry,
            };
            use $crate::processor::{BaseProcessor, StreamProcessor};
            use $crate::abi_stable::{
                erased_types::TD_Opaque,
                std_types::{RStr, RString, RSliceMut},
            };

            struct _FfiAdapter($ty);

            impl AbiStreamProcessor for _FfiAdapter {
                fn init(
                    &mut self,
                    uuid: RString,
                    ctx: $crate::processor::ProcessorContext,
                    analysis: AnalysisContextFFI,
                ) -> AnalysisContextFFI {
                    let mut native = $crate::analyzer::AnalysisContext::default();
                    analysis.drain_into(&mut native);
                    self.0.init(uuid.into(), &ctx, &mut native);
                    AnalysisContextFFI::from_context(&mut native)
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
                fn get_analysis_hash(&self) -> RString {
                    RString::from(self.0.get_analysis_hash())
                }
                fn process(
                    &mut self,
                    mut samples: RSliceMut<'_, f32>,
                    time: f64,
                    ctx: $crate::processor::ProcessorContext,
                ) {
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
                AbiStructuralProcessor, AbiStructuralProcessor_TO,
                AnalysisContextFFI, ProcessorDescriptorFFI, ProcessorEntry,
            };
            use $crate::processor::{BaseProcessor, StructuralProcessor};
            use $crate::abi_stable::{erased_types::TD_Opaque, std_types::{RStr, RString}};

            struct _FfiAdapter($ty);

            impl AbiStructuralProcessor for _FfiAdapter {
                fn init(
                    &mut self,
                    uuid: RString,
                    ctx: $crate::processor::ProcessorContext,
                    analysis: AnalysisContextFFI,
                ) -> AnalysisContextFFI {
                    let mut native = $crate::analyzer::AnalysisContext::default();
                    analysis.drain_into(&mut native);
                    self.0.init(uuid.into(), &ctx, &mut native);
                    AnalysisContextFFI::from_context(&mut native)
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
                fn get_analysis_hash(&self) -> RString {
                    RString::from(self.0.get_analysis_hash())
                }
                fn segments(
                    &self,
                    duration: f64,
                    ctx: $crate::processor::ProcessorContext,
                ) -> $crate::abi_stable::std_types::RVec<$crate::processor::Segment> {
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
                AbiAnalyzer, AbiAnalyzer_TO,
                AnalysisRequestFFI, AnalysisResultFFI,
                ProcessorDescriptorFFI, ProcessorEntry,
            };
            use $crate::analyzer::AudioAnalyser;
            use $crate::abi_stable::{
                erased_types::TD_Opaque,
                std_types::{ROption, RSlice},
            };

            struct _FfiAdapter($ty);

            impl AbiAnalyzer for _FfiAdapter {
                fn init(&mut self, request: AnalysisRequestFFI) {
                    let native = $crate::analyzer::AnalysisRequest::from(&request);
                    self.0.init(&native);
                }
                fn analyze(
                    &mut self,
                    samples:   RSlice<'_, f32>,
                    time:      f64,
                    exhausted: bool,
                    ctx:       $crate::processor::ProcessorContext,
                ) -> ROption<AnalysisResultFFI> {
                    match self.0.analyze(samples.as_slice(), time, exhausted, &ctx) {
                        Some((hash, boxed)) => ROption::RSome(AnalysisResultFFI::from_boxed(hash, &boxed)),
                        None => ROption::RNone,
                    }
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

    ($ty:ty, Stream, with: $an:ty = $id:expr) => {
        $crate::export_processor!($ty, Stream);
        $crate::__export_bundled_analyzer!($an, $id);
    };
    ($ty:ty, Structural, with: $an:ty = $id:expr) => {
        $crate::export_processor!($ty, Structural);
        $crate::__export_bundled_analyzer!($an, $id);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __export_bundled_analyzer {
    ($an:ty, $id:expr) => {
        const _: () = {
            use $crate::ffi::{
                AbiAnalyzer, AbiAnalyzer_TO,
                AnalysisRequestFFI, AnalysisResultFFI,
                AnalyzerDescriptorFFI,
            };
            use $crate::analyzer::AudioAnalyser;
            use $crate::abi_stable::{
                erased_types::TD_Opaque,
                std_types::{ROption, RSlice, RStr},
            };

            struct _FfiAnalyzerAdapter($an);

            impl AbiAnalyzer for _FfiAnalyzerAdapter {
                fn init(&mut self, request: AnalysisRequestFFI) {
                    let native = $crate::analyzer::AnalysisRequest::from(&request);
                    self.0.init(&native);
                }
                fn analyze(
                    &mut self,
                    samples:   RSlice<'_, f32>,
                    time:      f64,
                    exhausted: bool,
                    ctx:       $crate::processor::ProcessorContext,
                ) -> ROption<AnalysisResultFFI> {
                    match self.0.analyze(samples.as_slice(), time, exhausted, &ctx) {
                        Some((hash, boxed)) => ROption::RSome(AnalysisResultFFI::from_boxed(hash, &boxed)),
                        None => ROption::RNone,
                    }
                }
            }

            #[no_mangle]
            pub extern "C" fn musicum_analyzer_descriptor() -> &'static AnalyzerDescriptorFFI {
                static DESC: ::std::sync::OnceLock<AnalyzerDescriptorFFI> = ::std::sync::OnceLock::new();
                DESC.get_or_init(|| {
                    let id: &'static str = $id;
                    AnalyzerDescriptorFFI {
                        id:   RStr::from(id),
                        name: RStr::from(id),
                    }
                })
            }

            #[no_mangle]
            pub extern "C" fn musicum_analyzer_create()
                -> AbiAnalyzer_TO<'static, $crate::abi_stable::std_types::RBox<()>>
            {
                AbiAnalyzer_TO::from_value(
                    _FfiAnalyzerAdapter(<$an as ::std::default::Default>::default()),
                    TD_Opaque,
                )
            }
        };
    };
}
