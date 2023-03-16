use bevy::prelude::*;

use crate::uniforms::DepthMode;

/// A component for storing the computed depth at which the outline lies.
#[derive(Clone, Component, Default)]
pub struct ComputedOutlineDepth {
    pub(crate) world_origin: Vec3,
    pub(crate) depth_mode: DepthMode,
}

/// A component which specifies how the outline depth should be computed.
#[derive(Clone, Component)]
#[non_exhaustive]
pub enum SetOutlineDepth {
    /// A flat plane facing the camera and intersecting the specified point in model-space.
    Flat { model_origin: Vec3 },
    /// Real model-space.
    Real,
}

/// A component which specifies that this outline lies at the same depth as its parent.
#[derive(Clone, Component, Default)]
pub struct InheritOutlineDepth;

#[allow(clippy::type_complexity)]
pub(crate) fn compute_outline_depth(
    mut root_query: Query<
        (
            &mut ComputedOutlineDepth,
            &GlobalTransform,
            Changed<GlobalTransform>,
            Option<(&SetOutlineDepth, Changed<SetOutlineDepth>)>,
            Option<(&Children, Changed<Children>)>,
        ),
        Without<InheritOutlineDepth>,
    >,
    mut computed_query: Query<
        (&mut ComputedOutlineDepth, Changed<InheritOutlineDepth>),
        With<InheritOutlineDepth>,
    >,
    child_query: Query<(&Children, Changed<Children>)>,
) {
    for (mut computed, transform, changed_transform, set_depth, children) in root_query.iter_mut() {
        let mut changed =
            computed.is_added() || changed_transform || set_depth.filter(|(_, c)| *c).is_some();
        if changed {
            let (origin, depth_mode) = if let Some((sd, _)) = set_depth {
                match sd {
                    SetOutlineDepth::Flat {
                        model_origin: origin,
                    } => (*origin, DepthMode::Flat),
                    SetOutlineDepth::Real => (Vec3::NAN, DepthMode::Real),
                }
            } else {
                (Vec3::ZERO, DepthMode::Flat)
            };
            let matrix = transform.compute_matrix();
            computed.world_origin = matrix.project_point3(origin);
            computed.depth_mode = depth_mode;
        }
        if let Some((cs, changed_children)) = children {
            changed |= changed_children;
            for child in cs.iter() {
                propagate_outline_depth(
                    &computed,
                    changed,
                    *child,
                    &mut computed_query,
                    &child_query,
                );
            }
        }
    }
}

fn propagate_outline_depth(
    root_computed: &ComputedOutlineDepth,
    mut changed: bool,
    entity: Entity,
    computed_query: &mut Query<
        (&mut ComputedOutlineDepth, Changed<InheritOutlineDepth>),
        With<InheritOutlineDepth>,
    >,
    child_query: &Query<(&Children, Changed<Children>)>,
) {
    if let Ok((mut computed, changed_inherit)) = computed_query.get_mut(entity) {
        changed |= changed_inherit;
        if changed {
            *computed = root_computed.clone();
        }
        if let Ok((cs, changed_children)) = child_query.get(entity) {
            changed |= changed_children;
            for child in cs.iter() {
                propagate_outline_depth(
                    root_computed,
                    changed,
                    *child,
                    computed_query,
                    child_query,
                );
            }
        }
    }
}
