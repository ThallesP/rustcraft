//! net/minecraft/client/renderer/feature/submit/SubmitNode.java, with `BatchableSubmit` and
//! `TranslucentSubmit`.
//!
//! What a feature renderer is asked to draw. Java's submits are each renderer's own records; the
//! port's are the variants of one enum, of the kinds ported (an entity model's).

use std::rc::Rc;

use crate::net::minecraft::client::renderer::feature::feature_renderer_type::FeatureRendererType;
use crate::net::minecraft::client::renderer::feature::model_feature_renderer::ModelSubmit;
use crate::net::minecraft::util::mth::Mth;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::vector3f::Vector3f;

pub enum SubmitNode {
    Model(ModelSubmit),
}

impl SubmitNode {
    pub fn feature_type(&self) -> FeatureRendererType {
        match self {
            SubmitNode::Model(_) => FeatureRendererType::MODEL,
        }
    }

    /// `BatchableSubmit.batchKey()`, compared by identity, or `None` for a submit that isn't
    /// batchable.
    pub fn batch_key(&self) -> Option<usize> {
        match self {
            SubmitNode::Model(submit) => Some(Rc::as_ptr(&submit.render_type) as usize),
        }
    }

    /// `TranslucentSubmit.distanceToCameraSq()`.
    pub fn distance_to_camera_sq(&self) -> f32 {
        match self {
            SubmitNode::Model(submit) => compute_distance_to_camera_sq(submit.pose.pose()),
        }
    }
}

/// `TranslucentSubmit.computeDistanceToCameraSq(Matrix4fc)`: how far the pose's origin is.
pub fn compute_distance_to_camera_sq(pose: &Matrix4f) -> f32 {
    let mut translation = Vector3f::new();
    pose.get_translation(&mut translation);
    Mth::length_squared3_f32(translation.x, translation.y, translation.z)
}
