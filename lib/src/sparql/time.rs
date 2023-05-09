use ic_cdk;

/// Returns the Unix milliseconds in float64
pub fn now() -> f64 {
  (ic_cdk::api::time() / 1_000_000) as f64
}