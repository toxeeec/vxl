#define_import_path utils

fn rgb_to_gray(rgb: vec3f) -> f32 {
    return rgb.r * 0.299 + rgb.g * 0.587 + rgb.b * 0.114;
}
