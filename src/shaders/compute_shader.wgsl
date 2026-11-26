struct Camera {
    position: vec3<f32>,
    fov: f32,
    right: vec4<f32>,
    up: vec4<f32>,
    forward: vec4<f32>,
}

struct Ray {
    origin: vec3<f32>,
    dir: vec3<f32>,
}

struct Sphere {
    center: vec3<f32>,
    radius: f32,
    material_idx: u32,
}

struct Material {
    albedo: vec3<f32>
}

struct Light {
    color: vec3<f32>,
    intensity: f32,
    direction: vec3<f32>
}

struct Scene_Settings {
    sky_color: vec3<f32>,
    ambient: f32,
    sphere_count: u32,
    light_count: u32
}

struct Hit {
    // Distance along the ray, or NO_HIT
    t: f32,
    position: vec3<f32>,
    normal: vec3<f32>,
    material_idx: u32,
}

@group(0) @binding(0)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<storage, read> spheres: array<Sphere>;

@group(1) @binding(1)
var<storage, read> materials: array<Material>;

@group(1) @binding(2)
var<storage, read> lights: array<Light>;

@group(1) @binding(3)
var<uniform> scene_settings: Scene_Settings;

// Ray limits
const T_MIN: f32 = 0.001;
const T_MAX: f32 = 1e30;
const NO_HIT: f32 = -1.0;
const SHADOW_BIAS: f32 = 0.01;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let bounds = textureDimensions(output_texture);
    if global_id.x >= bounds.x || global_id.y >= bounds.y {
        return;
    }

    let ray = camera_ray(global_id.xy, bounds);
    let hit = closest_hit(ray);

    var color = scene_settings.sky_color;
    if hit.t != NO_HIT {
        color = shade(hit);
    }

    textureStore(output_texture, global_id.xy, vec4f(color, 1.0));
}

// Ray from the camera through the centre of the given pixel.
fn camera_ray(pixel: vec2<u32>, bounds: vec2<u32>) -> Ray {
    let aspect_ratio = f32(bounds.x) / f32(bounds.y);

    // Pixel centre in [0, 1], y pointing down
    let uv = (vec2f(pixel) + 0.5) / vec2f(bounds);

    // Point on the image plane at distance 1: [-1, 1] with y up,
    // scaled by the field of view and the aspect ratio
    var screen_pos = vec2f(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0) * tan(camera.fov / 2.0);
    screen_pos.x *= aspect_ratio;

    // x·right + y·up + forward
    let camera_ray = screen_pos.x * camera.right + screen_pos.y * camera.up + camera.forward;

    return Ray(camera.position, normalize(camera_ray.xyz));
}

// Nearest sphere hit along the ray. Returns a Hit with t = NO_HIT if nothing is hit.
fn closest_hit(ray: Ray) -> Hit {
    var closest_t = T_MAX;
    var closest_sphere: Sphere;
    var hit_anything = false;

    for (var i = 0u; i < scene_settings.sphere_count; i++) {
        // Passing closest_t as the upper limit only accepts hits nearer than the best so far
        let t = hit_sphere(spheres[i], ray, T_MIN, closest_t);
        if t != NO_HIT {
            closest_t = t;
            closest_sphere = spheres[i];
            hit_anything = true;
        }
    }

    if !hit_anything {
        return Hit(NO_HIT, vec3f(0.0), vec3f(0.0), 0);
    }

    let position = ray.origin + closest_t * ray.dir;
    let normal = (position - closest_sphere.center) / closest_sphere.radius;
    return Hit(closest_t, position, normal, closest_sphere.material_idx);
}

// True if anything blocks the path from `point` along `dir`.
// Stops at the first hit: a shadow ray only needs to know whether something is in the way.
fn in_shadow(point: vec3<f32>, dir: vec3<f32>) -> bool {
    let ray = Ray(point, dir);
    for (var i = 0u; i < scene_settings.sphere_count; i++) {
        if hit_sphere(spheres[i], ray, T_MIN, T_MAX) != NO_HIT {
            return true;
        }
    }
    return false;
}

fn shade(hit: Hit) -> vec3<f32> {

    let sphere_color = materials[hit.material_idx].albedo;

    var total_light = scene_settings.ambient * sphere_color;

    for (var i = 0u; i < scene_settings.light_count; i++){
        let current_light = lights[i];
        
        let diffuse = max(dot(hit.normal, current_light.direction), 0.0);

        // Start the shadow ray slightly off the surface so it does not hit its own sphere
        if (diffuse == 0 || in_shadow(hit.position + SHADOW_BIAS * hit.normal, current_light.direction)) {
            continue;
        }
        
        total_light += sphere_color * current_light.color * diffuse * current_light.intensity;
    }
    return total_light;
}

// Distance along the ray to the first intersection with the sphere in [t_min, t_max),
// or NO_HIT.
fn hit_sphere(s: Sphere, ray: Ray, t_min: f32, t_max: f32) -> f32 {
    let oc = ray.origin - s.center;
    let h = dot(ray.dir, oc);
    let c = dot(oc, oc) - s.radius * s.radius;
    let discriminant = h * h - c;

    if discriminant < 0.0 {
        return NO_HIT;
    }

    let sqd = sqrt(discriminant);

    var t = -h - sqd;
    if t >= t_min && t < t_max {
        return t;
    }

    t = -h + sqd;
    if t >= t_min && t < t_max {
        return t;
    }

    return NO_HIT;
}
