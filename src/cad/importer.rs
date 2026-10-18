use glam::Vec3;
use anyhow::{Result, Context};
use std::path::Path;

// opencascade-rs tipleri için soyutlama katmanı
// Gerçek ortamda crate: opencascade::{step::StepReader, brep::*, geom::*}
use crate::cad::b_rep_features::{CadFeature, FeatureType};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CadVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub feature_id: u32, // Raycast ve nesne seçimi için yüzey kimliği
}

pub struct TessellatedCadMesh {
    pub vertices: Vec<CadVertex>,
    pub indices: Vec<u32>,
    pub features: Vec<CadFeature>,
}

pub struct CadImporter;

impl CadImporter {
    /// STEP veya IGES dosyasını okur, B-Rep analizi yapar ve GPU mesh verisini çıkarır
    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<TessellatedCadMesh> {
        let path_ref = path.as_ref();
        let extension = path_ref.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        log::info!("CAD Modeli okunuyor: {:?}", path_ref);

        match extension.as_str() {
            "step" | "stp" => Self::import_step(path_ref),
            "iges" | "igs" => Self::import_iges(path_ref),
            _ => anyhow::bail!("Desteklenmeyen dosya formatı: {}", extension),
        }
    }

    fn import_step(path: &Path) -> Result<TessellatedCadMesh> {
        // 1. STEP Dosyasını oku
        // Gerçek implementasyonda:
        // let mut reader = STEPControl_Reader::new();
        // reader.read_file(path.to_str().unwrap())?;
        // reader.transfer_roots()?;
        // let shape = reader.one_shape()?;

        // 2. B-Rep Ağ Oluşturma (Tessellation)
        // BRepMesh_IncrementalMesh::new(&shape, linear_deflection: 0.05, is_relative: false, angular_deflection: 0.5);

        // 3. Yüzeyleri (Faces) Gezme ve Geometri Analizi
        Self::extract_brep_geometry()
    }

    fn import_iges(path: &Path) -> Result<TessellatedCadMesh> {
        // IGESControl_Reader pipeline
        Self::extract_brep_geometry()
    }

    /// Open CASCADE B-Rep yapısını tarayarak geometrik özellikleri ve üçgenleri üretir
    fn extract_brep_geometry() -> Result<TessellatedCadMesh> {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut features = Vec::new();

        // Örnek: Endüstriyel bir parçanın iki temel unsuru (Düzlem + Silindirik Delik)
        let plane_feat = CadFeature {
            id: 1,
            label: "UST_DUZLEM".to_string(),
            feature_type: FeatureType::Plane,
            nominal_center: Vec3::new(0.0, 0.0, 50.0),
            nominal_vector: Vec3::new(0.0, 0.0, 1.0),
            diameter: None,
            depth_or_length: None,
        };

        let hole_feat = CadFeature {
            id: 2,
            label: "MERKEZ_DELIK_Ø30".to_string(),
            feature_type: FeatureType::CylinderHole,
            nominal_center: Vec3::new(0.0, 0.0, 0.0),
            nominal_vector: Vec3::new(0.0, 0.0, 1.0),
            diameter: Some(30.0),
            depth_or_length: Some(50.0),
        };

        features.push(plane_feat);
        features.push(hole_feat);

        // Modelin görselleştirilmesi için örnek referans küp/yüzey mesh'i (Gerçek OCCT çıktısı burayı doldurur)
        Self::generate_placeholder_cad_mesh(&mut vertices, &mut indices);

        Ok(TessellatedCadMesh {
            vertices,
            indices,
            features,
        })
    }

    fn generate_placeholder_cad_mesh(vertices: &mut Vec<CadVertex>, indices: &mut Vec<u32>) {
        // Basit bir plaka geometrisi (100x100x50 mm)
        let half_w = 50.0;
        let half_l = 50.0;
        let h = 50.0;

        let base_vertices = [
            // Üst yüzey (Z = h)
            CadVertex { position: [-half_w, -half_l, h], normal: [0.0, 0.0, 1.0], feature_id: 1 },
            CadVertex { position: [ half_w, -half_l, h], normal: [0.0, 0.0, 1.0], feature_id: 1 },
            CadVertex { position: [ half_w,  half_l, h], normal: [0.0, 0.0, 1.0], feature_id: 1 },
            CadVertex { position: [-half_w,  half_l, h], normal: [0.0, 0.0, 1.0], feature_id: 1 },
        ];

        vertices.extend_from_slice(&base_vertices);
        indices.extend_from_slice(&[0, 1, 2, 2, 3, 0]);
    }
}
