use glam::Vec3;
use crate::cad::b_rep_features::{CadFeature, FeatureMeasurementPlan};
use crate::metrology::probe::CmmProbeAssembly;
use crate::metrology::path_planner::{PathPlanner, PathPlannerConfig};
use crate::metrology::collision::CollisionDetector;

pub struct AutoMeasurementProgram {
    pub plans: Vec<FeatureMeasurementPlan>,
    pub total_transit_path: Vec<Vec3>,
}

pub struct AutoPlanner;

impl AutoPlanner {
    /// Parçadaki tüm B-Rep unsurlarını sıralar ve kesintisiz tam otomatik ölçüm rotası üretir
    pub fn generate_full_inspection(
        mut features: Vec<CadFeature>,
        config: &PathPlannerConfig,
    ) -> AutoMeasurementProgram {
        let mut ordered_plans = Vec::new();
        let mut full_transit = Vec::new();
        let mut current_pos = Vec3::new(0.0, 0.0, config.safety_z);

        // 1. Unsur Sıralama (Greedy Nearest Neighbor - En Yakın Komşu Algoritması)
        while !features.is_empty() {
            let mut best_idx = 0;
            let mut min_dist = f32::MAX;

            for (i, feat) in features.iter().enumerate() {
                let dist = (feat.nominal_center - current_pos).length();
                if dist < min_dist {
                    min_dist = dist;
                    best_idx = i;
                }
            }

            let next_feature = features.remove(best_idx);

            // 2. Unsur için prob yönelimi ve temas yollarını hesapla
            let plan = PathPlanner::plan_feature(&next_feature, config);

            // 3. Önceki konumdan yeni unsura emniyet düzlemi üzerinden güvenli geçiş yolu bağla
            if let Some(first_touch) = plan.points.first() {
                let transit = CollisionDetector::generate_safe_transit_path(
                    current_pos,
                    first_touch.approach,
                    config.safety_z,
                );
                full_transit.extend(transit);
            }

            if let Some(last_touch) = plan.points.last() {
                current_pos = last_touch.retract;
            }

            ordered_plans.push(plan);
        }

        // Ölçüm bitiminde ana emniyet noktasına geri dönüş
        full_transit.push(Vec3::new(current_pos.x, current_pos.y, config.safety_z));

        AutoMeasurementProgram {
            plans: ordered_plans,
            total_transit_path: full_transit,
        }
    }
}
