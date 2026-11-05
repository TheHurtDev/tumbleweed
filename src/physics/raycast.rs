use hawthorn::math::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub source: Vec3,
    pub direction: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct RayIntersection {
    pub point: Vec3,
    pub distance: f32,
    pub normal: Vec3,
}

impl Ray {
    /// Returns the intersection between this ray and the plane defined by a point on
    /// the plane and its normal vector, or `None` if no intersection exists.
    pub fn intersect_plane(&self, point: Vec3, normal: Vec3) -> Option<RayIntersection> {
        let normal = normal.try_normalize()?;
        let direction = self.direction.try_normalize()?;

        let ray_normal_alignment = (-direction).dot(normal);

        const EPS: f32 = 1e-6;

        if ray_normal_alignment < EPS {
            return None;
        }

        let signed_plane_distance = (self.source - point).dot(normal);
        let distance_along_ray = signed_plane_distance / ray_normal_alignment;

        if distance_along_ray < 0.0 {
            return None;
        }

        Some(RayIntersection {
            point: self.source + direction * distance_along_ray,
            distance: distance_along_ray,
            normal,
        })
    }
}

/// Returns the closest point of intersection between `ray` and `triangles`, or
/// `None` if no intersection exists.
pub fn cast_ray(
    ray: Ray,
    triangles: impl Iterator<Item = (Vec3, Vec3, Vec3)>,
) -> Option<RayIntersection> {
    #[inline]
    fn is_inside_triangle(q: Vec3, n: Vec3, v: (Vec3, Vec3, Vec3)) -> bool {
        #[inline]
        fn is_inside_edge(q: Vec3, n: Vec3, v: (Vec3, Vec3)) -> bool {
            const EPS: f32 = 1e-6;

            (v.1 - v.0).cross(q - v.0).dot(n) >= -EPS
        }

        is_inside_edge(q, n, (v.0, v.1))
            && is_inside_edge(q, n, (v.1, v.2))
            && is_inside_edge(q, n, (v.2, v.0))
    }

    triangles
        .filter_map(|v| {
            let n = (v.1 - v.0).cross(v.2 - v.0);

            let intersection = ray.intersect_plane(v.0, n)?;

            if is_inside_triangle(intersection.point, intersection.normal, v) {
                Some(intersection)
            } else {
                None
            }
        })
        .min_by(|a, b| a.distance.total_cmp(&b.distance))
}
