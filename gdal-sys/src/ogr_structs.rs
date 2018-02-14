#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_snake_case)]
#[repr(C)]
pub struct OGREnvelope {
    pub MinX: f64,
    pub MaxX: f64,
    pub MinY: f64,
    pub MaxY: f64,
}

impl Default for OGREnvelope {
    fn default() -> Self {
        Self {
            MinX: 0.0,
            MaxX: 0.0,
            MinY: 0.0,
            MaxY: 0.0,
        }
    }
}
