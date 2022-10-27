//! Infrastructure for validating shapes
//!
//! Validation enforces various constraints about shapes and the objects that
//! constitute them. These constraints fall into 4 categories:
//!
//! - **Coherence:** Local forms of objects must be consistent with their
//!   canonical forms.
//! - **Geometric:** Comprises various object-specific constraints, for example
//!   edges or faces might not be allowed to intersect.
//! - **Structural:** All other objects that an object references must be part
//!   of the same shape.
//! - **Uniqueness:** Objects within a shape must be unique.
//!
//! Please note that not all of these validation categories are fully
//! implemented, as of this writing.

mod coherence;
mod curve;
mod cycle;
mod edge;
mod face;
mod shell;
mod sketch;
mod solid;
mod surface;
mod uniqueness;
mod vertex;

pub use self::{
    coherence::{CoherenceIssues, VertexCoherenceMismatch},
    uniqueness::UniquenessIssues,
};

use std::{collections::HashSet, convert::Infallible, ops::Deref};

use fj_math::Scalar;

use crate::iter::ObjectIters;

/// Validate an object
pub trait Validate: Sized {
    /// Validate the object using default configuration
    ///
    /// The following calls are equivalent:
    /// ``` rust
    /// # use fj_kernel::{
    /// #     objects::{GlobalVertex, Objects},
    /// #     validate::{Validate, ValidationConfig},
    /// # };
    /// # let objects = Objects::new();
    /// # let object = objects.global_vertices.insert(
    /// #     GlobalVertex::from_position([0., 0., 0.])
    /// # );
    /// object.validate();
    /// ```
    /// ``` rust
    /// # use fj_kernel::{
    /// #     objects::{GlobalVertex, Objects},
    /// #     validate::{Validate, ValidationConfig},
    /// # };
    /// # let objects = Objects::new();
    /// # let object = objects.global_vertices.insert(
    /// #     GlobalVertex::from_position([0., 0., 0.])
    /// # );
    /// object.validate_with_config(&ValidationConfig::default());
    /// ```
    fn validate(self) -> Result<Validated<Self>, ValidationError> {
        self.validate_with_config(&ValidationConfig::default())
    }

    /// Validate the object
    fn validate_with_config(
        self,
        config: &ValidationConfig,
    ) -> Result<Validated<Self>, ValidationError>;
}

impl<T> Validate for T
where
    T: for<'r> ObjectIters<'r>,
{
    fn validate_with_config(
        self,
        config: &ValidationConfig,
    ) -> Result<Validated<Self>, ValidationError> {
        let mut global_vertices = HashSet::new();

        for global_vertex in self.global_vertex_iter() {
            uniqueness::validate_vertex(
                global_vertex,
                &global_vertices,
                config.distinct_min_distance,
            )?;

            global_vertices.insert(*global_vertex);
        }
        for vertex in self.vertex_iter() {
            coherence::validate_vertex(vertex, config.identical_max_distance)?;
        }

        Ok(Validated(self))
    }
}

/// Validate an object
pub trait Validate2: Sized {
    /// The error that validation of the implementing type can result in
    type Error: Into<ValidationError>;

    /// Validate the object using default configuration
    fn validate(&self) -> Result<(), Self::Error> {
        self.validate_with_config(&ValidationConfig::default())
    }

    /// Validate the object
    fn validate_with_config(
        &self,
        config: &ValidationConfig,
    ) -> Result<(), Self::Error>;
}

/// Configuration required for the validation process
#[derive(Debug, Clone, Copy)]
pub struct ValidationConfig {
    /// The minimum distance between distinct objects
    ///
    /// Objects whose distance is less than the value defined in this field, are
    /// considered identical.
    pub distinct_min_distance: Scalar,

    /// The maximum distance between identical objects
    ///
    /// Objects that are considered identical might still have a distance
    /// between them, due to inaccuracies of the numerical representation. If
    /// that distance is less than the one defined in this field, can not be
    /// considered identical.
    pub identical_max_distance: Scalar,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            distinct_min_distance: Scalar::from_f64(5e-7), // 0.5 µm,

            // This value was chosen pretty arbitrarily. Seems small enough to
            // catch errors. If it turns out it's too small (because it produces
            // false positives due to floating-point accuracy issues), we can
            // adjust it.
            identical_max_distance: Scalar::from_f64(5e-14),
        }
    }
}

/// Wrapper around an object that indicates the object has been validated
///
/// Returned by implementations of `Validate`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Validated<T>(T);

impl<T> Validated<T> {
    /// Consume this instance of `Validated` and return the wrapped object
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Validated<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// An error that can occur during a validation
#[allow(clippy::large_enum_variant)]
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// Coherence validation failed
    #[error("Coherence validation failed")]
    Coherence(#[from] CoherenceIssues),

    /// Geometric validation failed
    #[error("Geometric validation failed")]
    Geometric,

    /// Uniqueness validation failed
    #[error("Uniqueness validation failed")]
    Uniqueness(#[from] UniquenessIssues),
}

impl From<Infallible> for ValidationError {
    fn from(infallible: Infallible) -> Self {
        match infallible {}
    }
}

#[cfg(test)]
mod tests {
    use fj_interop::ext::ArrayExt;
    use fj_math::{Point, Scalar};

    use crate::{
        objects::{
            Curve, GlobalCurve, GlobalEdge, GlobalVertex, HalfEdge, Objects,
            SurfaceVertex, Vertex,
        },
        partial::HasPartial,
        path::SurfacePath,
        validate::{Validate, ValidationConfig, ValidationError},
    };

    #[test]
    fn coherence_edge() -> anyhow::Result<()> {
        let objects = Objects::new();

        let surface = objects.surfaces.xy_plane();

        let points_surface = [[0., 0.], [1., 0.]];
        let points_global = [[0., 0., 0.], [1., 0., 0.]];

        let curve = {
            let path = SurfacePath::line_from_points(points_surface);
            let global_form = objects.global_curves.insert(GlobalCurve)?;

            objects.curves.insert(Curve::new(
                surface.clone(),
                path,
                global_form,
            ))?
        };

        let vertices_global = points_global.try_map_ext(|point| {
            objects
                .global_vertices
                .insert(GlobalVertex::from_position(point))
        })?;

        let [a_surface, b_surface] = points_surface
            .zip_ext(vertices_global)
            .try_map_ext(|(point_surface, vertex_global)| {
                objects.surface_vertices.insert(SurfaceVertex::new(
                    point_surface,
                    surface.clone(),
                    vertex_global,
                ))
            })?;

        let deviation = Scalar::from_f64(0.25);

        let a = objects.vertices.insert(Vertex::new(
            Point::from([Scalar::ZERO + deviation]),
            curve.clone(),
            a_surface,
        ))?;
        let b = objects.vertices.insert(Vertex::new(
            Point::from([Scalar::ONE]),
            curve.clone(),
            b_surface,
        ))?;
        let vertices = [a, b];

        let global_edge = GlobalEdge::partial()
            .from_curve_and_vertices(&curve, &vertices)
            .build(&objects)?;
        let half_edge = objects
            .half_edges
            .insert(HalfEdge::new(vertices, global_edge));

        let result =
            half_edge.clone().validate_with_config(&ValidationConfig {
                identical_max_distance: deviation * 2.,
                ..ValidationConfig::default()
            });
        assert!(result.is_ok());

        let result = half_edge.validate_with_config(&ValidationConfig {
            identical_max_distance: deviation / 2.,
            ..ValidationConfig::default()
        });
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn uniqueness_vertex() -> anyhow::Result<()> {
        let objects = Objects::new();
        let mut shape = Vec::new();

        let deviation = Scalar::from_f64(0.25);

        let a = Point::from([0., 0., 0.]);

        let mut b = a;
        b.x += deviation;

        let config = ValidationConfig {
            distinct_min_distance: deviation * 2.,
            ..ValidationConfig::default()
        };

        // Adding a vertex should work.
        shape.push(
            objects
                .global_vertices
                .insert(GlobalVertex::from_position(a)),
        );
        shape.clone().validate_with_config(&config)?;

        // Adding a second vertex that is considered identical should fail.
        shape.push(
            objects
                .global_vertices
                .insert(GlobalVertex::from_position(b)),
        );
        let result = shape.validate_with_config(&config);
        assert!(matches!(result, Err(ValidationError::Uniqueness(_))));

        Ok(())
    }
}