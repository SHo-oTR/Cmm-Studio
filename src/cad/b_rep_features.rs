use glam::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub enum FeatureType {
    Plane,
    CylinderHole, // İç çap (delik)
    CylinderBoss, // Dış çap (pim/şaft)
    Sphere,
    Cone,
}

#[derive(Debug, Clone)]
pub struct CadFeature {
    pub id: usize,
    pub label: String,
    pub feature_type: FeatureType,
    pub nominal_center: Vec3,
    pub nominal_vector: Vec3, // Yüzey veya eksen normali (Birim vektör)
    pub diameter: Option<f32>,
    pub depth_or_length: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct MeasurementPoint {
    pub target: Vec3,      // Parçaya dokunulacak nominal nokta
    pub normal: Vec3,      // Temas anındaki yüzey normali (Vektör İçi/Dışı)
    pub approach: Vec3,    // Güvenli yaklaşma noktası
    pub retract: Vec3,     // Geri çekilme noktası
}

#[derive(Debug, Clone)]
pub struct FeatureMeasurementPlan {
    pub feature: CadFeature,
    pub points: Vec<MeasurementPoint>,
    pub safety_clearance_z: f32,
}
