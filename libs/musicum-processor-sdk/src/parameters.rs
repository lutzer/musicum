
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
    Int   { id: &'static str, name: &'static str, default: i32, min: i32, max: i32, editable: bool },
    Canvas {
        id:           &'static str,
        name:         &'static str,
        aspect_ratio: f32,
    },
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
    pub fn new(default: f32, min: f32, max: f32) -> Self {
        FloatParam {
            value: default.clamp(min, max),
            min,
            max,
        }
    }

    pub fn get(&self) -> f32 {
        self.value
    }

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
    pub fn new(default: bool) -> Self {
        BoolParam { value: default }
    }

    pub fn get(&self) -> f32 {
        if self.value {
            1.0
        } else {
            0.0
        }
    }

    pub fn get_bool(&self) -> bool {
        self.value
    }

    pub fn set(&mut self, v: bool) {
        self.value = v;
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

/// Runtime struct for Time Param
#[derive(Default)]
pub struct TimeParam {
    value: f64
}

impl TimeParam {
    pub fn new(default: f64) -> Self {
        TimeParam {
            value: default
        }
    }

    pub fn get(&self) -> f64 {
        self.value
    }

    pub fn set(&mut self, v: f64) {
        self.value = v;
    }
}

impl FromParamInfo for TimeParam {
    fn from_param_info(info: &ProcessorParamaterInfo) -> Option<Self> {
        match info {
            ProcessorParamaterInfo::Time { default, ..} => {
                Some(TimeParam::new(*default))
            }
            _ => None,
        }
    }
}

/// Runtime struct for Int Param
#[derive(Default)]
pub struct IntParam {
    value: i32,
    min: i32,
    max: i32,
}

impl IntParam {
    pub fn new(default: i32, min: i32, max: i32) -> Self {
        IntParam {
            value: default.clamp(min, max),
            min,
            max,
        }
    }

    pub fn get(&self) -> i32 {
        self.value
    }

    pub fn set(&mut self, v: i32) {
        self.value = v.clamp(self.min, self.max);
    }
}

impl FromParamInfo for IntParam {
    fn from_param_info(info: &ProcessorParamaterInfo) -> Option<Self> {
        match info {
            ProcessorParamaterInfo::Int { default, min, max, ..} => {
                Some(IntParam::new(*default, *min, *max))
            }
            _ => None,
        }
    }
}