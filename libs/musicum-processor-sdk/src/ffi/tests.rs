use crate::analyzer::{AnalysisContext, AnalysisRequest, AnalysisResult};
use crate::ffi::{AnalysisContextFFI, AnalysisRequestFFI, AnalysisResultFFI};
use std::any::Any;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct DummyResult { pub value: f32 }

#[typetag::serde]
impl AnalysisResult for DummyResult {
    fn as_any(&self) -> &dyn Any { self }
}

#[test]
fn analysis_request_roundtrips_through_ffi() {
    let original = AnalysisRequest {
        analyzer_id: "rms",
        hash: "abc123".to_string(),
        params: vec![("window".to_string(), 0.05_f64)],
    };
    let ffi = AnalysisRequestFFI::from(&original);
    let restored = AnalysisRequest::from(&ffi);
    assert_eq!(restored.analyzer_id, "rms");
    assert_eq!(restored.hash, "abc123");
    assert_eq!(restored.params, vec![("window".to_string(), 0.05_f64)]);
}

#[test]
fn analysis_result_roundtrips_through_ffi() {
    let boxed: Box<dyn AnalysisResult> = Box::new(DummyResult { value: 0.42 });
    let ffi = AnalysisResultFFI::from_boxed("h1".to_string(), &boxed);
    assert_eq!(ffi.hash.as_str(), "h1");
    let restored: Box<dyn AnalysisResult> = ffi.into_boxed().expect("decode");
    let cast = restored.as_any().downcast_ref::<DummyResult>().expect("type");
    assert!((cast.value - 0.42).abs() < 1e-6);
}

#[test]
fn analysis_context_roundtrips_through_ffi() {
    let mut ctx = AnalysisContext::default();
    ctx.results.insert(
        "h1".to_string(),
        Box::new(DummyResult { value: 1.0 }) as Box<dyn AnalysisResult>,
    );
    ctx.requests.push(AnalysisRequest {
        analyzer_id: "rms",
        hash: "h2".to_string(),
        params: vec![],
    });

    let ffi = AnalysisContextFFI::from_context(&mut ctx);
    let mut restored = AnalysisContext::default();
    ffi.drain_into(&mut restored);

    let r = restored.get_result::<DummyResult>(&"h1".to_string()).expect("present");
    assert!((r.value - 1.0).abs() < 1e-6);
    assert_eq!(restored.requests.len(), 1);
    assert_eq!(restored.requests[0].hash, "h2");
}
