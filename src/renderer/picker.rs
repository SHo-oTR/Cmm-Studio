use glam::{Vec2, Vec3, Mat4};
use crate::cad::b_rep_features::CadFeature;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

pub struct RaycastPicker;

impl RaycastPicker {
    pub fn new() -> Self {
        Self
    }

    /// Ekran pikselini (X, Y) 3D dünya uzayında fırlatılan bir ışına dönüştürür
    pub fn screen_point_to_ray(
        screen_pos: Vec2,
        viewport_size: Vec2,
        inv_view_proj: Mat4,
    ) -> Ray {
        // Normalize Device Coordinates (NDC) [-1, 1]
        let ndc_x = (2.0 * screen_pos.x) / viewport_size.x - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y) / viewport_size.y;

        let near_clip = Vec3::new(ndc_x, ndc_y, 0.0);
        let far_clip = Vec3::new(ndc_x, ndc_y, 1.0);

        let near_world = inv_view_proj.project_point3(near_clip);
        let far_world = inv_view_proj.project_point3(far_clip);

        let direction = (far_world - near_world).normalize();

        Ray {
            origin: near_world,
            direction,
        }
    }

    /// Işının model üzerindeki unsurlarla kesişimini kontrol eder ve en yakın unsuru döndürür
    pub fn pick_feature<'a>(ray: &Ray, features: &'a [CadFeature]) -> Option<&'a CadFeature> {
        let mut closest_feature = None;
        let mut min_dist = f32::MAX;

        for feat in features {
            // Işın - Düzlem / Silindir kesişim yaklaşımı
            let to_center = feat.nominal_center - ray.origin;
            let projection = to_center.dot(ray.direction);

            if projection > 0.0 {
                let closest_point_on_ray = ray.origin + ray.direction * projection;
                let distance_to_feature = (closest_point_on_ray - feat.nominal_center).length();

                // Tolerans eşiği (örneğin 15 mm yarıçapında tıklama alanı)
                if distance_to_feature < 15.0 && projection < min_dist {
                    min_dist = projection;
                    closest_feature = Some(feat);
                }
            }
        }

        closest_feature
    }
}
