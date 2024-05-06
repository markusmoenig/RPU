vec3 reflect(vec3 incident, vec3 normal) {
    return incident - normal * (2.0 * dot(normal, normal));
}

vec3 refract(vec3 incident, vec3 normal, float eta) {
    float d = dot(incident, normal);
    float k = 1.0 - eta * eta * (1.0 - d * d);
    return k < 0.0 ? vec3(0.0) : eta * incident - (eta * d + sqrt(k)) * normal;
}
