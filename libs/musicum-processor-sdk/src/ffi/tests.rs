use crate::analyzer::{AnalysisRequest, AnalysisResult};
use crate::ffi::{AnalysisRequestFFI, AnalysisResultFFI};
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
        slot_key: 0xdead_beef,
        params: vec![("window".to_string(), 0.05_f64)],
    };
    let ffi = AnalysisRequestFFI::from(&original);
    let restored = AnalysisRequest::from(&ffi);
    assert_eq!(restored.analyzer_id, "rms");
    assert_eq!(restored.slot_key, 0xdead_beef);
    assert_eq!(restored.params, vec![("window".to_string(), 0.05_f64)]);
}

#[test]
fn analysis_result_roundtrips_through_ffi() {
    let boxed: Box<dyn AnalysisResult> = Box::new(DummyResult { value: 0.42 });
    let ffi = AnalysisResultFFI::from_boxed(&boxed);
    let restored: Box<dyn AnalysisResult> = ffi.into_boxed().expect("decode");
    let cast = restored.as_any().downcast_ref::<DummyResult>().expect("type");
    assert!((cast.value - 0.42).abs() < 1e-6);
}
