use std::fmt::Write;
use chrono::Local;
use crate::cad::b_rep_features::{FeatureMeasurementPlan, FeatureType};
use crate::metrology::probe::CmmProbeAssembly;

impl crate::dmis::writer::DmisWriter {
    /// Tam otomatik çoklu kafa açılı ve emniyetli DMIS programı üretir
    pub fn generate_advanced_dmis(
        plans: &[FeatureMeasurementPlan],
        probe: &CmmProbeAssembly,
    ) -> String {
        let mut d = String::new();
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        writeln!(&mut d, "$$ ==========================================").unwrap();
        writeln!(&mut d, "$$ CMM STUDIO - ENDUSTRIYEL METROLOJI CIKTISI").unwrap();
        writeln!(&mut d, "$$ Tarih: {}", now).unwrap();
        writeln!(&mut d, "$$ ==========================================").unwrap();
        writeln!(&mut d, "DMISMN/'AUTO_CMM_JOB', 05.2").unwrap();
        writeln!(&mut d, "UNITS/MM, ANGDEC").unwrap();
        writeln!(&mut d, "DECPL/ALL, 4").unwrap();
        writeln!(&mut d, "MODE/AUTO, PROG, MAN").unwrap();
        writeln!(&mut d, "PRCOMP/ON").unwrap();
        writeln!(&mut d, "FLY/1.0").unwrap(); // Köşe yuvarlatma (yumuşak hareket)
        writeln!(&mut d, "$$").unwrap();

        // Prob Tanımlama
        writeln!(
            &mut d,
            "SNSDEF/PROBE1, FIX, CART, 0.0, 0.0, -{:.4}, 0.0, 0.0, 1.0, {:.4}",
            probe.stylus_length, probe.ruby_ball_diameter
        ).unwrap();
        writeln!(&mut d, "LOADVS/PROBE1").unwrap();
        writeln!(&mut d, "$$").unwrap();

        let mut current_a = -1.0;
        let mut current_b = -1.0;

        for (idx, plan) in plans.iter().enumerate() {
            let feat_name = format!("F_{}_{}", plan.feature.label, idx + 1);

            // Gerekliyse prob kafasını yeni açıya çevir (PH10 indeksleme)
            let optimal_angles = CmmProbeAssembly::calculate_optimal_orientation(plan.feature.nominal_vector);
            if (optimal_angles.a_deg - current_a).abs() > 0.1 || (optimal_angles.b_deg - current_b).abs() > 0.1 {
                current_a = optimal_angles.a_deg;
                current_b = optimal_angles.b_deg;
                writeln!(&mut d, "SNSLCT/SA({:.1}),SB({:.1})", current_a, current_b).unwrap();
            }

            // Emniyet düzlemine çıkış
            writeln!(
                &mut d,
                "GOTO/CART, {:.4}, {:.4}, {:.4}",
                plan.feature.nominal_center.x,
                plan.feature.nominal_center.y,
                plan.safety_clearance_z
            ).unwrap();

            match plan.feature.feature_type {
                FeatureType::CylinderHole => {
                    let dia = plan.feature.diameter.unwrap_or(0.0);
                    let c = plan.feature.nominal_center;
                    let v = plan.feature.nominal_vector;

                    writeln!(
                        &mut d,
                        "F({}) = FEAT/CIRCLE, INNER, CART, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}",
                        feat_name, c.x, c.y, c.z, v.x, v.y, v.z, dia
                    ).unwrap();

                    writeln!(&mut d, "MEAS/CIRCLE, F({}), {}", feat_name, plan.points.len()).unwrap();
                    for pt in &plan.points {
                        writeln!(
                            &mut d,
                            "PTMEAS/CART, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}",
                            pt.target.x, pt.target.y, pt.target.z, pt.normal.x, pt.normal.y, pt.normal.z
                        ).unwrap();
                    }
                    writeln!(&mut d, "ENDMES").unwrap();
                }
                FeatureType::Plane => {
                    let c = plan.feature.nominal_center;
                    let v = plan.feature.nominal_vector;

                    writeln!(
                        &mut d,
                        "F({}) = FEAT/PLANE, CART, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}",
                        feat_name, c.x, c.y, c.z, v.x, v.y, v.z
                    ).unwrap();

                    writeln!(&mut d, "MEAS/PLANE, F({}), {}", feat_name, plan.points.len()).unwrap();
                    for pt in &plan.points {
                        writeln!(
                            &mut d,
                            "PTMEAS/CART, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}",
                            pt.target.x, pt.target.y, pt.target.z, pt.normal.x, pt.normal.y, pt.normal.z
                        ).unwrap();
                    }
                    writeln!(&mut d, "ENDMES").unwrap();
                }
                _ => {}
            }
            writeln!(&mut d, "$$").unwrap();
        }

        writeln!(&mut d, "ENDFIL").unwrap();
        d
    }
}
