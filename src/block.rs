use bevy::prelude::{Bundle, Transform, Vec3, Visibility};

#[derive(Bundle)]
pub(crate) struct BlockBundle {
    transform: Transform,
    visibility: Visibility,
}

impl BlockBundle {
    pub(crate) fn new(pos: Vec3, is_visible: bool) -> Self {
        Self {
            transform: Transform::from_translation(pos),
            visibility: if is_visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        }
    }
}
