use glam::Vec3;
use std::f32::consts::PI;
use crate::cad::b_rep_features::{CadFeature, FeatureType, MeasurementPoint, FeatureMeasurementPlan};

pub struct PathPlannerConfig {
    pub touch_count: usize,
    pub approach_distance: f32,
    pub retract_distance: f32,
    pub safety_z: f32,
    pub probe_radius: f32, // Yakut yarıçapı kompanzasyonu için
}

pub struct PathPlanner;

impl PathPlanner {
    /// Seçilen geometriye göre yarı-otomatik temas noktalarını ve emniyet yollarını üretir
    pub fn plan_feature(feature: &CadFeature, config: &PathPlannerConfig) -> FeatureMeasurementPlan {
        let mut points = Vec::new();

        match feature.feature_type {
            FeatureType::CylinderHole => {
                let radius = feature.diameter.unwrap_or(10.0) / 2.0;
                let center = feature.nominal_center;
                let axis = feature.nominal_vector.normalize();

                // Silindir eksenine dik iki ortonormal referans vektör bul (u, v)
                let ortho = if axis.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
                let u = axis.cross(ortho).normalize();
                let v = axis.cross(u).normalize();

                let step_angle = 2.0 * PI / (config.touch_count as f32);

                for i in 0..config.touch_count {
                    let theta = i as f32 * step_angle;
                    let dir = u * theta.cos() + v * theta.sin();

                    // Delik içi olduğu için temas yüzeyi dışa, yüzey normali ise merkeze dönüktür
                    let target = center + dir * radius;
                    let normal = -dir; // Deliğin iç yüzeyinden merkeze doğru normal

                    let approach = target - dir * config.approach_distance;
                    let retract = target - dir * config.retract_distance;

                    points.push(MeasurementPoint {
                        target,
                        normal,
                        approach,
                        retract,
                    });
                }
            }
            FeatureType::Plane => {
                let center = feature.nominal_center;
                let normal = feature.nominal_vector.normalize();

                // Düzlem için ızgara şeklinde minimum 3 veya 4 nokta dağılımı
                let span = 15.0; // mm
                let ortho = if normal.z.abs() < 0.9 { Vec3::Z } else { Vec3::X };
                let u = normal.cross(ortho).normalize();
                let v = normal.cross(u).normalize();

                let offsets = [
                    (-span, -span),
                    (span, -span),
                    (span, span),
                    (-span, span),
                ];

                let count = config.touch_count.min(offsets.len());
                for (du, dv) in offsets.into_iter().take(count) {
                    let target = center + u * du + v * dv;
                    let approach = target + normal * config.approach_distance;
                    let retract = target + normal * config.retract_distance;

                    points.push(MeasurementPoint {
                        target,
                        normal,
                        approach,
                        retract,
                    });
                }
            }
            _ => {
                // Diğer unsurlar için tekil temas
                let target = feature.nominal_center;
                let normal = feature.nominal_vector.normalize();
                points.push(MeasurementPoint {
                    target,
                    normal,
                    approach: target + normal * config.approach_distance,
                    retract: target + normal * config.retract_distance,
                });
            }
        }

        FeatureMeasurementPlan {
            feature: feature.clone(),
            points,
            safety_clearance_z: config.safety_z,
        }
    }
}
