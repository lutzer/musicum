//! ABI-safe representations of the analysis types.

use abi_stable::{
    std_types::{RStr, RString, RVec, Tuple2},
    StableAbi,
};

use crate::analyzer::{AnalysisContext, AnalysisRequest, AnalysisResult};

#[repr(C)]
#[derive(StableAbi, Clone)]
pub struct AnalysisRequestFFI {
    pub analyzer_id: RStr<'static>,
    pub hash:        RString,
    pub params:      RVec<Tuple2<RString, f64>>,
}

impl From<&AnalysisRequest> for AnalysisRequestFFI {
    fn from(r: &AnalysisRequest) -> Self {
        Self {
            analyzer_id: RStr::from(r.analyzer_id),
            hash:        RString::from(r.hash.as_str()),
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
            hash: r.hash.as_str().to_owned(),
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
    pub hash:  RString,
    /// bincode-serialized `Box<dyn AnalysisResult>` (typetag dispatch).
    pub bytes: RVec<u8>,
}

impl AnalysisResultFFI {
    #[allow(clippy::borrowed_box)]
    pub fn from_boxed(hash: String, boxed: &Box<dyn AnalysisResult>) -> Self {
        let bytes = bincode::serialize(boxed)
            .expect("AnalysisResult must be serializable");
        Self {
            hash: RString::from(hash),
            bytes: bytes.into(),
        }
    }

    pub fn into_boxed(self) -> Option<Box<dyn AnalysisResult>> {
        bincode::deserialize::<Box<dyn AnalysisResult>>(self.bytes.as_slice()).ok()
    }
}

#[repr(C)]
#[derive(StableAbi, Clone)]
pub struct AnalysisContextFFI {
    pub requests: RVec<AnalysisRequestFFI>,
    pub results:  RVec<AnalysisResultFFI>,
}

impl AnalysisContextFFI {
    /// Serializes everything in `ctx` into ABI-safe form.
    /// `ctx.requests` is *moved* into the ffi struct (cleared on `ctx`)
    /// so the host can later see only newly-appended requests after a
    /// dylib `init` call. `ctx.results` is borrowed and serialized;
    /// each result carries its own hash.
    pub fn from_context(ctx: &mut AnalysisContext) -> Self {
        let requests = std::mem::take(&mut ctx.requests)
            .iter().map(AnalysisRequestFFI::from).collect();
        let mut results = RVec::new();
        for (hash, boxed) in ctx.results.iter() {
            results.push(AnalysisResultFFI::from_boxed(hash.clone(), boxed));
        }
        Self { requests, results }
    }

    /// Drains this FFI snapshot into `ctx` on the receiving side.
    /// Existing entries in `ctx.results` are preserved; matching hashes
    /// are overwritten. `ctx.requests` is extended with the carried
    /// requests.
    pub fn drain_into(self, ctx: &mut AnalysisContext) {
        for ffi_req in self.requests.into_iter() {
            ctx.requests.push(AnalysisRequest::from(&ffi_req));
        }
        for ffi_res in self.results.into_iter() {
            let hash: String = ffi_res.hash.clone().into();
            if let Some(boxed) = ffi_res.into_boxed() {
                ctx.results.insert(hash, boxed);
            }
        }
    }
}

#[repr(C)]
#[derive(StableAbi, Clone)]
pub struct AnalyzerDescriptorFFI {
    pub id:   RStr<'static>,
    pub name: RStr<'static>,
}
