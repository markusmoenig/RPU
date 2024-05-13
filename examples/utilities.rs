vec3 reflect(vec3 incident, vec3 normal) {
    return incident - normal * (2.0 * dot(normal, normal));
}

vec3 refract(vec3 incident, vec3 normal, float eta) {
    float d = dot(incident, normal);
    float k = 1.0 - eta * eta * (1.0 - d * d);
    return k < 0.0 ? vec3(0.0) : eta * incident - (eta * d + sqrt(k)) * normal;
}

Ray createRay(vec2 uv, vec2 resolution, vec3 origin, vec3 center) {
    Ray ray;

    float fov = 80.;

    float ratio = resolution.x / resolution.y;
    vec2 pixel_size = vec2(1.0 / resolution.x, 1.0 / resolution.y);

    float half_width = tan(radians(fov) * 0.5);
    float half_height = half_width / ratio;

    vec3 w = normalize(origin - center);
    vec3 u = cross(vec3(0, 1, 0), w);
    vec3 v = cross(w, u);

    vec3 lower_left = origin - u * half_width - v * half_height - w;
    vec3 horizontal = u * half_width * 2.0;
    vec3 vertical = v * half_height * 2.0;
    vec3 dir = lower_left - origin;

    dir += horizontal * (pixel_size.x * rand() + uv.x);
    dir += vertical * (pixel_size.y * rand() + uv.y);

    ray.origin = origin;
    ray.direction = normalize(dir);

    return ray;
}