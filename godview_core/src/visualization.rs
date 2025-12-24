//! Visualization module for GodView using Rerun.io
//!
//! This module provides real-time visualization of:
//! - 6D Gaussian uncertainty ellipsoids (position + velocity covariance)
//! - Agent communication packets
//! - Highlander CRDT merge events
//! - Trust verification status
//!
//! Enable with the `visualization` feature flag.

use crate::godview_tracking::GlobalHazardPacket;
use nalgebra::{Matrix3, Matrix6};
use rerun::{RecordingStream, RecordingStreamBuilder};
use uuid::Uuid;

/// Rerun-based visualizer for GodView distributed sensor fusion
pub struct RerunVisualizer {
    rec: RecordingStream,
}

impl RerunVisualizer {
    /// Create a new visualizer that spawns the Rerun viewer
    pub fn new(app_id: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let rec = RecordingStreamBuilder::new(app_id)
            .spawn()?;
        
        // Log initial setup
        rec.log_static(
            "world",
            &rerun::ViewCoordinates::RIGHT_HAND_Z_UP,
        )?;
        
        Ok(Self { rec })
    }
    
    /// Create a visualizer that saves to a file (for web sharing)
    pub fn new_to_file(app_id: &str, path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let rec = RecordingStreamBuilder::new(app_id)
            .save(path)?;
        
        rec.log_static(
            "world",
            &rerun::ViewCoordinates::RIGHT_HAND_Z_UP,
        )?;
        
        Ok(Self { rec })
    }
    
    /// Log a track with its 6D Gaussian uncertainty ellipsoid
    pub fn log_track(
        &self,
        track_id: Uuid,
        position: [f64; 3],
        velocity: [f64; 3],
        covariance: &Matrix6<f64>,
        entity_type: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Extract position covariance (upper-left 3x3)
        let pos_cov: Matrix3<f64> = covariance.fixed_view::<3, 3>(0, 0).into();
        
        // Eigen decomposition for ellipsoid axes
        let eigen = pos_cov.symmetric_eigen();
        let half_sizes: [f32; 3] = [
            (eigen.eigenvalues[0].abs().sqrt() * 2.0) as f32, // 2-sigma
            (eigen.eigenvalues[1].abs().sqrt() * 2.0) as f32,
            (eigen.eigenvalues[2].abs().sqrt() * 2.0) as f32,
        ];
        
        // Calculate rotation quaternion from eigenvectors
        let rotation = nalgebra::UnitQuaternion::from_matrix(&eigen.eigenvectors);
        let quat = rotation.as_ref();
        
        let path = format!("world/tracks/{}", track_id);
        
        // Log the uncertainty ellipsoid
        self.rec.log(
            format!("{}/ellipsoid", path),
            &rerun::Ellipsoids3D::from_centers_and_half_sizes(
                [[position[0] as f32, position[1] as f32, position[2] as f32]],
                [half_sizes],
            )
            .with_quaternions([[quat.w as f32, quat.i as f32, quat.j as f32, quat.k as f32]])
            .with_colors([[0, 255, 200, 80]]) // Cyan with transparency
            .with_fill_mode(rerun::FillMode::Solid)
        )?;
        
        // Log the center point
        self.rec.log(
            format!("{}/center", path),
            &rerun::Points3D::new([[position[0] as f32, position[1] as f32, position[2] as f32]])
                .with_colors([[255, 255, 255, 255]]) // White
                .with_radii([0.1])
        )?;
        
        // Log velocity vector
        let vel_magnitude = (velocity[0].powi(2) + velocity[1].powi(2) + velocity[2].powi(2)).sqrt();
        if vel_magnitude > 0.01 {
            self.rec.log(
                format!("{}/velocity", path),
                &rerun::Arrows3D::from_vectors([[
                    velocity[0] as f32,
                    velocity[1] as f32,
                    velocity[2] as f32,
                ]])
                .with_origins([[position[0] as f32, position[1] as f32, position[2] as f32]])
                .with_colors([[255, 200, 0, 255]]) // Yellow
            )?;
        }
        
        // Log entity type as text
        self.rec.log(
            format!("{}/label", path),
            &rerun::TextLog::new(format!("{}: {}", entity_type, &track_id.to_string()[..8]))
        )?;
        
        Ok(())
    }
    
    /// Log a simplified track from a hazard packet
    pub fn log_packet_detection(
        &self,
        packet: &GlobalHazardPacket,
        uncertainty: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = format!("world/detections/{}", packet.entity_id);
        
        self.rec.log(
            path,
            &rerun::Points3D::new([[
                packet.position[0] as f32,
                packet.position[1] as f32,
                packet.position[2] as f32,
            ]])
            .with_colors([[255, 100, 100, 200]]) // Red-ish
            .with_radii([uncertainty])
        )?;
        
        Ok(())
    }
    
    /// Log a data packet traveling between agents
    pub fn log_data_packet(
        &self,
        from: [f64; 3],
        to: [f64; 3],
        packet_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.rec.log(
            format!("world/packets/{}", packet_id),
            &rerun::Arrows3D::from_vectors([[
                (to[0] - from[0]) as f32,
                (to[1] - from[1]) as f32,
                (to[2] - from[2]) as f32,
            ]])
            .with_origins([[from[0] as f32, from[1] as f32, from[2] as f32]])
            .with_colors([[0, 212, 255, 200]]) // Cyan
        )?;
        
        Ok(())
    }
    
    /// Log a Highlander CRDT merge event
    pub fn log_highlander_merge(
        &self,
        old_id: Uuid,
        new_id: Uuid,
        num_sources: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.rec.log(
            "logs/crdt",
            &rerun::TextLog::new(format!(
                "🏆 HIGHLANDER: {} → {} ({} sources merged)",
                &old_id.to_string()[..8],
                &new_id.to_string()[..8],
                num_sources
            ))
        )?;
        
        Ok(())
    }
    
    /// Log trust verification status
    pub fn log_trust_event(
        &self,
        agent_id: &str,
        verified: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let status = if verified { "✓ VERIFIED" } else { "✗ REJECTED" };
        let color = if verified { "green" } else { "red" };
        
        self.rec.log(
            "logs/trust",
            &rerun::TextLog::new(format!("🔐 {}: {} ({})", agent_id, status, color))
        )?;
        
        Ok(())
    }
    
    /// Log H3 spatial cell activation
    pub fn log_h3_cell(
        &self,
        cell_index: u64,
        center: [f64; 3],
        active: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let color = if active { [0, 255, 136, 100] } else { [50, 50, 50, 50] };
        
        self.rec.log(
            format!("world/h3/{:x}", cell_index),
            &rerun::Points3D::new([[center[0] as f32, center[1] as f32, center[2] as f32]])
                .with_colors([color])
                .with_radii([1.0])
        )?;
        
        Ok(())
    }
    
    /// Log uncertainty reduction stats
    pub fn log_stats(
        &self,
        total_tracks: usize,
        avg_uncertainty: f64,
        reduction_percent: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.rec.log(
            "stats/tracks",
            &rerun::Scalar::new(total_tracks as f64),
        )?;
        
        self.rec.log(
            "stats/uncertainty",
            &rerun::Scalar::new(avg_uncertainty),
        )?;
        
        self.rec.log(
            "stats/reduction",
            &rerun::Scalar::new(reduction_percent),
        )?;
        
        Ok(())
    }
    
    /// Log a ground plane grid for scene context
    pub fn log_ground_plane(&self, size: f32, divisions: usize) -> Result<(), Box<dyn std::error::Error>> {
        let mut points = Vec::new();
        let step = size / divisions as f32;
        
        // Create grid points
        for i in 0..=divisions {
            let coord = -size / 2.0 + i as f32 * step;
            // Along X
            points.push([coord, -size / 2.0, 0.0]);
            points.push([coord, size / 2.0, 0.0]);
            // Along Y
            points.push([-size / 2.0, coord, 0.0]);
            points.push([size / 2.0, coord, 0.0]);
        }
        
        self.rec.log_static(
            "world/ground/grid",
            &rerun::LineStrips3D::new(
                points.chunks(2).map(|c| c.to_vec()).collect::<Vec<_>>()
            )
            .with_colors([[60, 60, 60, 100]]) // Dark gray
        )?;
        
        Ok(())
    }
    
    /// Log an agent (vehicle/drone) as a 3D box at a position
    pub fn log_agent(
        &self,
        agent_name: &str,
        position: [f64; 3],
        size: [f32; 3], // [length, width, height]
        color: [u8; 4],
        is_drone: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let height_offset = if is_drone { position[2] } else { size[2] as f64 / 2.0 };
        
        self.rec.log(
            format!("world/agents/{}", agent_name),
            &rerun::Boxes3D::from_centers_and_sizes(
                [[position[0] as f32, position[1] as f32, height_offset as f32]],
                [size],
            )
            .with_colors([color])
            .with_labels([agent_name])
        )?;
        
        Ok(())
    }
    
    /// Log road segments for scene context
    pub fn log_road(&self, from: [f32; 2], to: [f32; 2], width: f32) -> Result<(), Box<dyn std::error::Error>> {
        // Calculate road direction
        let dx = to[0] - from[0];
        let dy = to[1] - from[1];
        let length = (dx * dx + dy * dy).sqrt();
        let center_x = (from[0] + to[0]) / 2.0;
        let center_y = (from[1] + to[1]) / 2.0;
        
        // Rotation angle
        let angle = dy.atan2(dx);
        let quat = nalgebra::UnitQuaternion::from_euler_angles(0.0, 0.0, angle as f64);
        let q = quat.as_ref();
        
        self.rec.log_static(
            format!("world/roads/{:.0}_{:.0}_{:.0}_{:.0}", from[0], from[1], to[0], to[1]),
            &rerun::Boxes3D::from_centers_and_sizes(
                [[center_x, center_y, 0.01]], // Slightly above ground
                [[length, width, 0.02]],
            )
            .with_quaternions([[q.w as f32, q.i as f32, q.j as f32, q.k as f32]])
            .with_colors([[40, 40, 45, 255]]) // Dark asphalt gray
        )?;
        
        Ok(())
    }
    
    /// Log a track with custom color for the ellipsoid
    pub fn log_track_colored(
        &self,
        track_id: Uuid,
        position: [f64; 3],
        velocity: [f64; 3],
        covariance: &Matrix6<f64>,
        label: &str,
        color: [u8; 4],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Extract position covariance (upper-left 3x3)
        let pos_cov: Matrix3<f64> = covariance.fixed_view::<3, 3>(0, 0).into();
        
        // Eigen decomposition for ellipsoid axes
        let eigen = pos_cov.symmetric_eigen();
        let half_sizes: [f32; 3] = [
            (eigen.eigenvalues[0].abs().sqrt() * 2.0) as f32, // 2-sigma
            (eigen.eigenvalues[1].abs().sqrt() * 2.0) as f32,
            (eigen.eigenvalues[2].abs().sqrt() * 2.0) as f32,
        ];
        
        // Calculate rotation quaternion from eigenvectors
        let rotation = nalgebra::UnitQuaternion::from_matrix(&eigen.eigenvectors);
        let quat = rotation.as_ref();
        
        let path = format!("world/tracks/{}", label.replace(" ", "_"));
        
        // Log the uncertainty ellipsoid with custom color
        self.rec.log(
            format!("{}/ellipsoid", path),
            &rerun::Ellipsoids3D::from_centers_and_half_sizes(
                [[position[0] as f32, position[1] as f32, position[2] as f32]],
                [half_sizes],
            )
            .with_quaternions([[quat.w as f32, quat.i as f32, quat.j as f32, quat.k as f32]])
            .with_colors([color])
            .with_fill_mode(rerun::FillMode::Solid)
        )?;
        
        // Log the center point
        self.rec.log(
            format!("{}/center", path),
            &rerun::Points3D::new([[position[0] as f32, position[1] as f32, position[2] as f32]])
                .with_colors([[255, 255, 255, 255]]) // White
                .with_radii([0.08])
        )?;
        
        // Log velocity vector
        let vel_magnitude = (velocity[0].powi(2) + velocity[1].powi(2) + velocity[2].powi(2)).sqrt();
        if vel_magnitude > 0.01 {
            self.rec.log(
                format!("{}/velocity", path),
                &rerun::Arrows3D::from_vectors([[
                    velocity[0] as f32,
                    velocity[1] as f32,
                    velocity[2] as f32,
                ]])
                .with_origins([[position[0] as f32, position[1] as f32, position[2] as f32]])
                .with_colors([[255, 200, 0, 255]]) // Yellow
            )?;
        }
        
        Ok(())
    }
    
    /// Set the current timestamp for timeline scrubbing
    pub fn set_time(&self, name: &str, timestamp_ms: u64) {
        self.rec.set_time_nanos(name, timestamp_ms as i64 * 1_000_000);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[ignore] // Requires Rerun viewer
    fn test_visualizer_creation() {
        let viz = RerunVisualizer::new("test_app");
        assert!(viz.is_ok());
    }
}
