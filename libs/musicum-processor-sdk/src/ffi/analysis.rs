//! ABI-safe representations of the analysis types.

use abi_stable::{
    StableAbi, sabi_trait, std_types::{ROption, RSlice, RStr, RString, RVec, Tuple2},
};

use crate::{AudioAnalyser, analyzer::{AnalysisRequest, AnalysisResult}, ffi::ProcessorContextFFI};

#[repr(C)]
#[derive(StableAbi, Clone)]
pub struct AnalysisRequestFFI {
    pub analyzer_id: RStr<'static>,
    pub hash:    u64,
    pub params:      RVec<Tuple2<RString, f64>>,
}

impl From<&AnalysisRequest> for AnalysisRequestFFI {
    fn from(r: &AnalysisRequest) -> Self {
        Self {
            analyzer_id: RStr::from(r.analyzer_id),
            hash:    r.hash,
            params: r.params.iter()
                .map(|(k, v)| Tuple2(RString::from(k.as_str()), *v))
                .collect(),
        }
    }
}

impl From<&AnalysisRequestFFI> for AnalysisRequest {
    fn from(r: &AnalysisRequestFFI) -> Self {
        Self {
            analyzer_id: leak_static_str(r.analyzer_id.as_str()),
            hash:    r.hash,
            params: r.params.iter()
                .map(|Tuple2(k, v)| (k.as_str().to_owned(), *v))
                .collect(),
        }
    }
}

/// `AnalysisRequest::analyzer_id` is `&'static str` — when reconstructing
/// across the FFI, we must intern the id into a `'static` string.
/// One-time leak per unique id; acceptable because analyzer ids are a
/// small bounded set per process.
fn leak_static_str(s: &str) -> &'static str {
    Box::leak(s.to_owned().into_boxed_str())
}

#[repr(C)]
#[derive(StableAbi, Clone)]
pub struct AnalysisResultFFI {
    /// bincode-serialized `Box<dyn AnalysisResult>` (typetag dispatch).
    pub bytes: RVec<u8>,
}

impl AnalysisResultFFI {
    pub fn from_boxed(boxed: &dyn AnalysisResult) -> Self {
        let bytes = bincode::serialize(boxed)
            .expect("AnalysisResult must be serializable");
        Self { bytes: bytes.into() }
    }

    pub fn into_boxed(self) -> Option<Box<dyn AnalysisResult>> {
        bincode::deserialize::<Box<dyn AnalysisResult>>(self.bytes.as_slice()).ok()
    }
}

// ── ABI-safe analyzer trait + generic adapter ─────────────────────────────────

#[sabi_trait]
pub trait AbiAnalyzer: Send + Sync {
    fn init(&mut self, request: AnalysisRequestFFI);
    fn analyze(
        &mut self,
        samples:   RSlice<'_, f32>,
        time:      f64,
        exhausted: bool,
        ctx:       ProcessorContextFFI,
    ) -> ROption<AnalysisResultFFI>;
}

pub struct FfiAnalyzerAdapter<T: AudioAnalyser>(pub T);

impl<T: AudioAnalyser + Send + Sync> AbiAnalyzer for FfiAnalyzerAdapter<T> {
    fn init(&mut self, request: AnalysisRequestFFI) {
        let native = AnalysisRequest::from(&request);
        self.0.init(&native);
    }
    fn analyze(
        &mut self,
        samples:   RSlice<'_, f32>,
        time:      f64,
        exhausted: bool,
        ctx:       ProcessorContextFFI,
    ) -> ROption<AnalysisResultFFI> {
        match self.0.analyze(samples.as_slice(), time, exhausted, &ctx) {
            Some(boxed) => ROption::RSome(AnalysisResultFFI::from_boxed(&*boxed)),
            None        => ROption::RNone,
        }
    }
}
