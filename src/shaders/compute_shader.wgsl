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
    color: vec3<f32>
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

// Ray limits
const T_MIN: f32 = 0.001;
const T_MAX: f32 = 1e30;
const NO_HIT: f32 = -1.0;
const SHADOW_BIAS: f32 = 0.01;

// Lighting. All colours here are linear; the comment gives the sRGB value
// they were picked as, converted with the exact sRGB curve.
const SKY_COLOR = vec3f(0.0844, 0.6376, 0.8228); // sRGB (82, 209, 234)
const LIGHT_DIR = normalize(vec3f(0.0, 3.0, 3.0)); // from the surface towards the light
const LIGHT_COLOR = vec3f(1.0, 0.95, 0.9);
const AMBIENT: f32 = 0.05;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let bounds = textureDimensions(output_texture);
    if global_id.x >= bounds.x || global_id.y >= bounds.y {
        return;
    }

    let ray = camera_ray(global_id.xy, bounds);
    let hit = closest_hit(ray);

    var color = SKY_COLOR;
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

    for (var i = 0u; i < arrayLength(&spheres); i++) {
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
    for (var i = 0u; i < arrayLength(&spheres); i++) {
        if hit_sphere(spheres[i], ray, T_MIN, T_MAX) != NO_HIT {
            return true;
        }
    }
    return false;
}

// Lambert diffuse with one directional light, plus ambient. Returns linear colour.
fn shade(hit: Hit) -> vec3<f32> {

    let sphere_color = materials[hit.material_idx].color;

    let ambient = AMBIENT * sphere_color;

    // Start the shadow ray slightly off the surface so it does not hit its own sphere
    if in_shadow(hit.position + SHADOW_BIAS * hit.normal, LIGHT_DIR) {
        return ambient;
    }

    let diffuse = max(dot(hit.normal, LIGHT_DIR), 0.0);
    return sphere_color * LIGHT_COLOR * diffuse + ambient;
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
