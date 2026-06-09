
pub enum ProcessorParamaterInfo {
    Float {
        id:      &'static str,
        name:    &'static str,
        default: f32,
        min:     f32,
        max:     f32,
        step:    f32,
        unit:    &'static str,
        editable: bool
    },
    Bool  { id: &'static str, name: &'static str, default: bool, editable: bool },
    Time  { id: &'static str, name: &'static str, default: f64, editable: bool },
    Int   { id: &'static str, name: &'static str, default: i64, min: i64, max: i64, editable: bool },
}

impl ProcessorParamaterInfo {

    pub fn get_param<T: FromParamInfo>(&self) -> Option<T> {
        T::from_param_info(self)
    }
}

pub trait FromParamInfo: Sized {
    fn from_param_info(info: &ProcessorParamaterInfo) -> Option<Self>;
}

/// Runtime struct for Float Param
#[derive(Default)]
pub struct FloatParam {
    value: f32,
    min: f32,
    max: f32,
}

impl FloatParam {
    /// Create a new holder initialised to `default`, clamped to `[min, max]`.
    pub fn new(default: f32, min: f32, max: f32) -> Self {
        FloatParam {
            value: default.clamp(min, max),
            min,
            max,
        }
    }

    /// Return the current value.
    pub fn get(&self) -> f32 {
        self.value
    }

    /// Set a new value, clamping to `[min, max]`.
    pub fn set(&mut self, v: f32) {
        self.value = v.clamp(self.min, self.max);
    }
}

impl FromParamInfo for FloatParam {
    fn from_param_info(info: &ProcessorParamaterInfo) -> Option<Self> {
        match info {
            ProcessorParamaterInfo::Float { default, min, max, ..} => {
                Some(FloatParam::new(*default, *min, *max))
            }
            _ => None,
        }
    }
}

// Runtime struct for Bool Param
#[derive(Default)]
pub struct BoolParam {
    value: bool,
}

impl BoolParam {
    /// Create a new holder initialised to `default`.
    pub fn new(default: bool) -> Self {
        BoolParam { value: default }
    }

    /// Return `1.0` if true, `0.0` if false.
    pub fn get(&self) -> f32 {
        if self.value {
            1.0
        } else {
            0.0
        }
    }

    /// Return the raw `bool` value.
    pub fn get_bool(&self) -> bool {
        self.value
    }

    /// Set from an `f32`: any non-zero value is `true`.
    pub fn set(&mut self, v: f32) {
        self.value = v != 0.0;
    }
}

impl FromParamInfo for BoolParam {
    fn from_param_info(info: &ProcessorParamaterInfo) -> Option<Self> {
        match info {
            ProcessorParamaterInfo::Bool { default, ..} => {
                Some(BoolParam::new(*default))
            }
            _ => None,
        }
    }
}