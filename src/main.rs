mod cad;
mod metrology;
mod dmis;

slint::include_modules!();

use std::fs::File;
use std::io::Write;
use glam::Vec3;
use cad::b_rep_features::{CadFeature, FeatureType, FeatureMeasurementPlan};
use metrology::path_planner::{PathPlanner, PathPlannerConfig};
use dmis::writer::DmisWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let app = MainWindow::new()?;
    let app_weak = app.as_weak();

    // Örnek aktif parça unsuru (CAD motoru entegre edilene kadar demonstrasyon için)
    let current_feature = CadFeature {
        id: 1,
        label: "DELIK_SILINDIR_25".to_string(),
        feature_type: FeatureType::CylinderHole,
        nominal_center: Vec3::new(100.0, 50.0, -10.0),
        nominal_vector: Vec3::new(0.0, 0.0, 1.0),
        diameter: Some(25.0),
        depth_or_length: Some(20.0),
    };

    // Arayüz ilk verilerini doldur
    app.set_feature_name(current_feature.label.clone().into());
    app.set_feature_type("İç Silindir (Delik)".into());
    app.set_nominal_coords(
        format!(
            "X: {:.2} | Y: {:.2} | Z: {:.2}",
            current_feature.nominal_center.x,
            current_feature.nominal_center.y,
            current_feature.nominal_center.z
        ).into(),
    );
    app.set_nominal_size(format!("Çap: {:.2} mm", current_feature.diameter.unwrap_or(0.0)).into());

    // 1. "Model Yükle" Buton Eylemi
    let _app_weak_cad = app_weak.clone();
    app.on_open_cad_file(move || {
        if let Some(app) = _app_weak_cad.upgrade() {
            // İleride buraya native dosya açma dialoğu (rfd kütüphanesi) eklenecek
            app.set_status_text("STEP/IGES yükleme motoru hazırlandı.".into());
        }
    });

    // 2. "Ölçüm Yolunu Hesapla" Buton Eylemi
    let active_feat = current_feature.clone();
    let app_weak_plan = app_weak.clone();
    app.on_generate_path(move || {
        if let Some(app) = app_weak_plan.upgrade() {
            let pts_count = app.get_touch_points() as usize;

            let config = PathPlannerConfig {
                touch_count: pts_count,
                approach_distance: 5.0,
                retract_distance: 3.0,
                safety_z: 50.0,
                probe_radius: 2.0,
            };

            let plan = PathPlanner::plan_feature(&active_feat, &config);
            app.set_status_text(
                format!("Başarılı: {} için {} ölçüm noktası ve yolu üretildi.", plan.feature.label, plan.points.len()).into()
            );
        }
    });

    // 3. "DMIS Dışa Aktar" Buton Eylemi
    let active_feat_dmis = current_feature.clone();
    let app_weak_dmis = app_weak.clone();
    app.on_export_dmis(move || {
        if let Some(app) = app_weak_dmis.upgrade() {
            let config = PathPlannerConfig {
                touch_count: app.get_touch_points() as usize,
                approach_distance: 5.0,
                retract_distance: 3.0,
                safety_z: 50.0,
                probe_radius: 2.0,
            };

            let plan = PathPlanner::plan_feature(&active_feat_dmis, &config);
            let dmis_code = DmisWriter::generate_dmis(&[plan]);

            // Çıktıyı dosyaya kaydet
            let filename = "olcum_programi.dmi";
            if let Ok(mut file) = File::create(filename) {
                let _ = file.write_all(dmis_code.as_bytes());
                app.set_status_text(format!("DMIS dosyası kaydedildi: {}", filename).into());
            }
        }
    });

    app.run()?;
    Ok(())
}
