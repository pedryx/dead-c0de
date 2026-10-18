use bevy::prelude::*;

/// Pixels the pointer must move while pressed before a click becomes a drag.
const DRAG_THRESHOLD: f32 = 8.;

pub fn plugin(app: &mut App) {
    app.add_observer(start_drag)
        .add_observer(track_drag)
        .add_observer(end_drag);
}

/// Can be dragged with the pointer.
#[derive(Component, Default, Clone)]
pub struct Draggable {
    /// Position when grabbed, zero for UI nodes.
    pub start: Vec2,
    /// How far the pointer moved since the grab, in pixels.
    pub distance: Vec2,
}

impl Draggable {
    /// Whether the pointer moved far enough to drag rather than click.
    pub fn dragging(&self) -> bool {
        self.distance.length() > DRAG_THRESHOLD
    }
}

/// Starts a drag.
fn start_drag(
    drag: On<PointerDragStart>,
    mut commands: Commands,
    mut draggables: Query<(&mut Draggable, Option<&Transform>)>,
) {
    if let Ok((mut draggable, transform)) = draggables.get_mut(drag.entity) {
        draggable.start =
            transform.map_or(Vec2::ZERO, |transform| transform.translation.truncate());
        commands.entity(drag.entity).insert(Pickable::IGNORE);
    }
}

/// Tracks how far the entity was dragged.
fn track_drag(drag: On<PointerDrag>, mut draggables: Query<&mut Draggable>) {
    if let Ok(mut draggable) = draggables.get_mut(drag.entity) {
        draggable.distance = drag.distance;
    }
}

/// Ends a drag.
fn end_drag(
    drag: On<PointerDragEnd>,
    mut commands: Commands,
    mut draggables: Query<&mut Draggable>,
) {
    if let Ok(mut draggable) = draggables.get_mut(drag.entity) {
        draggable.distance = Vec2::ZERO;
        commands.entity(drag.entity).remove::<Pickable>();
    }
}
