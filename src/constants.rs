use std::collections::HashMap;

lazy_static::lazy_static! {
    pub static ref CONSTANTS: HashMap<String, f64> = {
        let mut m = HashMap::new();
        m.insert("pi".to_string(), std::f64::consts::PI);
        m.insert("π".to_string(), std::f64::consts::PI);
        m.insert("e".to_string(), std::f64::consts::E);
        m.insert("phi".to_string(), 1.618033988749895);
        m.insert("φ".to_string(), 1.618033988749895);
        m.insert("tau".to_string(), std::f64::consts::TAU);
        m.insert("τ".to_string(), std::f64::consts::TAU);
        m.insert("sqrt2".to_string(), std::f64::consts::SQRT_2);
        m.insert("√2".to_string(), std::f64::consts::SQRT_2);
        m.insert("sqrt3".to_string(), 1.7320508075688772);
        m.insert("√3".to_string(), 1.7320508075688772);
        m.insert("golden".to_string(), 1.618033988749895);
        m
    };
}

#[allow(dead_code)]
pub fn get_constant(name: &str) -> Option<f64> {
    CONSTANTS.get(name).copied()
}

#[allow(dead_code)]
pub fn is_constant(name: &str) -> bool {
    CONSTANTS.contains_key(name)
}
