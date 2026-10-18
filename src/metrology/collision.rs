use glam::Vec3;
use crate::metrology::probe::CmmProbeAssembly;
use crate::cad::importer::CadVertex;

#[derive(Debug, Clone)]
pub struct CollisionCapsule {
    pub start: Vec3, // Mil başlangıcı (Kafa bağlantısı)
    pub end: Vec3,   // Mil sonu (Yakut merkez)
    pub radius: f32, // Yarıçap + Emniyet payı
}

pub struct CollisionDetector;

impl CollisionDetector {
    /// Bir çizgi segmenti (prob mili) ile bir nokta (CAD üçgen köşesi) arasındaki minimum mesafeyi hesaplar
    pub fn distance_segment_to_point(a: Vec3, b: Vec3, p: Vec3) -> f32 {
        let ab = b - a;
        let ap = p - a;
        let t = (ap.dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
        let closest = a + ab * t;
        (p - closest).length()
    }

    /// Verilen prob pozisyonunda model ile temas/çarpışma olup olmadığını test eder
    pub fn is_probe_colliding(
        probe: &CmmProbeAssembly,
        ruby_center: Vec3,
        cad_vertices: &[CadVertex],
        safety_margin: f32,
    ) -> bool {
        let probe_dir = probe.get_direction_vector();
        let shank_top = ruby_center - probe_dir * probe.stylus_length;

        let total_radius = (probe.stylus_diameter / 2.0) + safety_margin;

        // Modelin köşe noktalarına karşı hızlı kapsül testi
        for v in cad_vertices {
            let v_pos = Vec3::from_array(v.position);
            let dist = Self::distance_segment_to_point(shank_top, ruby_center, v_pos);
            if dist < total_radius {
                return true; // Çarpışma riski tespit edildi!
            }
        }
        false
    }

    /// İki nokta arasındaki hareketi emniyet düzlemine (Clearance Plane) çıkararak güvenli hale getirir
    pub fn generate_safe_transit_path(
        start: Vec3,
        end: Vec3,
        clearance_z: f32,
    ) -> Vec<Vec3> {
        let mut path = Vec::new();
        path.push(start);

        // Eğer başlangıç veya hedef emniyet seviyesinin altındaysa Z emniyet düzlemine çık
        if start.z < clearance_z || end.z < clearance_z {
            path.push(Vec3::new(start.x, start.y, clearance_z));
            path.push(Vec3::new(end.x, end.y, clearance_z));
        }

        path.push(end);
        path
    }
}
