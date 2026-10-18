use glam::{Mat4, Quat, Vec3};

#[derive(Debug, Clone)]
pub struct Ph10Angles {
    pub a_deg: f32, // 0.0 to 105.0 in 7.5 deg increments
    pub b_deg: f32, // -180.0 to +180.0 in 7.5 deg increments
}

impl Ph10Angles {
    pub fn new(a: f32, b: f32) -> Result<Self, &'static str> {
        let a_clamped = (a / 7.5).round() * 7.5;
        let b_clamped = (b / 7.5).round() * 7.5;

        if !(0.0..=105.0).contains(&a_clamped) {
            return Err("A açısı 0° ile 105° arasında olmalıdır.");
        }
        if !(-180.0..=180.0).contains(&b_clamped) {
            return Err("B açısı -180° ile +180° arasında olmalıdır.");
        }

        Ok(Self {
            a_deg: a_clamped,
            b_deg: b_clamped,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CmmProbeAssembly {
    pub head_type: String,       // Örn: "Renishaw PH10M"
    pub angles: Ph10Angles,      // Aktif A/B Açıları
    pub stylus_length: f32,      // Mil uzunluğu (mm), örn: 50.0
    pub stylus_diameter: f32,    // Mil gövde çapı (mm), örn: 1.5
    pub ruby_ball_diameter: f32, // Yakut uç çapı (mm), örn: 4.0
}

impl CmmProbeAssembly {
    pub fn standard_renishaw_4mm() -> Self {
        Self {
            head_type: "Renishaw PH10M".to_string(),
            angles: Ph10Angles::new(0.0, 0.0).unwrap(),
            stylus_length: 50.0,
            stylus_diameter: 1.5,
            ruby_ball_diameter: 4.0,
        }
    }

    pub fn ruby_radius(&self) -> f32 {
        self.ruby_ball_diameter / 2.0
    }

    /// Hedef yüzey normaline göre en uygun PH10 A/B açı kombinasyonunu seçer
    pub fn calculate_optimal_orientation(surface_normal: Vec3) -> Ph10Angles {
        // Prob yaklaşma vektörü yüzey normalinin tersidir
        let approach_dir = -surface_normal.normalize();

        // Z ekseni ile açı (A açısı)
        let z_axis = Vec3::Z;
        let cos_a = approach_dir.dot(-z_axis).clamp(-1.0, 1.0);
        let a_deg = cos_a.acos().to_degrees();

        // XY düzlemindeki azimut açısı (B açısı)
        let b_deg = approach_dir.y.atan2(approach_dir.x).to_degrees();

        Ph10Angles::new(a_deg, b_deg).unwrap_or(Ph10Angles {
            a_deg: 0.0,
            b_deg: 0.0,
        })
    }

    /// Probun yönelim vektörünü (Birim Vektör) hesaplar
    pub fn get_direction_vector(&self) -> Vec3 {
        let rot_z = Quat::from_rotation_z(self.angles.b_deg.to_radians());
        let rot_y = Quat::from_axis_angle(rot_z * Vec3::Y, self.angles.a_deg.to_radians());
        
        // Varsayılan kafa yönü aşağıya doğrudur (-Z)
        (rot_y * rot_z * -Vec3::Z).normalize()
    }
}
