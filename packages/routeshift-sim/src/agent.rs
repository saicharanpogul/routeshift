use crate::types::AgentState;

/// An agent (driver) in the simulation.
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: u32,
    pub origin: u32,
    pub destination: u32,
    /// Sequence of edge IDs forming the route
    pub route: Vec<u32>,
    /// Length of each edge in the route (km)
    pub route_edge_lengths: Vec<f64>,
    /// Current state
    pub state: AgentState,
    /// Whether this agent follows system-suggested route
    pub compliant: bool,
    /// Whether this is the player
    pub is_player: bool,
}

impl Agent {
    pub fn new(
        id: u32,
        origin: u32,
        destination: u32,
        route: Vec<u32>,
        route_edge_lengths: Vec<f64>,
        compliant: bool,
        is_player: bool,
    ) -> Self {
        let state = if route.is_empty() {
            AgentState::Idle
        } else {
            AgentState::Traveling {
                edge_index: 0,
                progress_km: 0.0,
            }
        };

        Agent {
            id,
            origin,
            destination,
            route,
            route_edge_lengths,
            state,
            compliant,
            is_player,
        }
    }

    /// Advance the agent by dt seconds. Returns true if the agent just arrived.
    ///
    /// `edge_speeds_kmh[edge_id]` gives the current speed on that edge in km/h.
    pub fn tick(&mut self, dt_seconds: f64, edge_speeds_kmh: &[f64]) -> bool {
        match &mut self.state {
            AgentState::Traveling {
                edge_index,
                progress_km,
            } => {
                if *edge_index >= self.route.len() {
                    self.state = AgentState::Arrived;
                    return true;
                }

                let edge_id = self.route[*edge_index] as usize;
                let speed_kmh = edge_speeds_kmh.get(edge_id).copied().unwrap_or(30.0);
                let speed_kms = speed_kmh / 3600.0; // km per second
                let distance = speed_kms * dt_seconds;

                *progress_km += distance;

                // Check if we've passed the end of the current edge
                let edge_length = self.route_edge_lengths[*edge_index];
                while *progress_km >= edge_length {
                    *progress_km -= edge_length;
                    *edge_index += 1;

                    if *edge_index >= self.route.len() {
                        self.state = AgentState::Arrived;
                        return true;
                    }
                }

                false
            }
            _ => false,
        }
    }

    /// Get the current edge ID the agent is on, or None if not traveling.
    pub fn current_edge(&self) -> Option<u32> {
        match &self.state {
            AgentState::Traveling { edge_index, .. } => {
                self.route.get(*edge_index).copied()
            }
            _ => None,
        }
    }

    /// Get progress as fraction (0.0 to 1.0) of total route.
    pub fn route_progress(&self) -> f64 {
        match &self.state {
            AgentState::Traveling {
                edge_index,
                progress_km,
            } => {
                let total: f64 = self.route_edge_lengths.iter().sum();
                if total <= 0.0 {
                    return 0.0;
                }
                let completed: f64 = self.route_edge_lengths[..*edge_index].iter().sum();
                (completed + progress_km) / total
            }
            AgentState::Arrived => 1.0,
            AgentState::Idle => 0.0,
        }
    }

    /// Get current speed in km/h based on the edge the agent is on.
    pub fn current_speed(&self, edge_speeds_kmh: &[f64]) -> f64 {
        match self.current_edge() {
            Some(eid) => edge_speeds_kmh.get(eid as usize).copied().unwrap_or(0.0),
            None => 0.0,
        }
    }

    /// Estimate remaining time in seconds.
    pub fn eta_seconds(&self, edge_speeds_kmh: &[f64]) -> f64 {
        match &self.state {
            AgentState::Traveling {
                edge_index,
                progress_km,
            } => {
                let mut total_seconds = 0.0;

                // Remaining on current edge
                let current_len = self.route_edge_lengths[*edge_index];
                let remaining = (current_len - progress_km).max(0.0);
                let current_edge_id = self.route[*edge_index] as usize;
                let speed = edge_speeds_kmh
                    .get(current_edge_id)
                    .copied()
                    .unwrap_or(30.0);
                if speed > 0.0 {
                    total_seconds += remaining / (speed / 3600.0);
                }

                // Subsequent edges
                for i in (*edge_index + 1)..self.route.len() {
                    let eid = self.route[i] as usize;
                    let len = self.route_edge_lengths[i];
                    let spd = edge_speeds_kmh.get(eid).copied().unwrap_or(30.0);
                    if spd > 0.0 {
                        total_seconds += len / (spd / 3600.0);
                    }
                }

                total_seconds
            }
            _ => 0.0,
        }
    }

    /// Interpolate position along the route given edge geometries.
    ///
    /// `edge_geometries[edge_id]` = Vec of [lng, lat] coords for that edge.
    pub fn position(&self, edge_geometries: &[Vec<[f64; 2]>]) -> [f64; 2] {
        match &self.state {
            AgentState::Traveling {
                edge_index,
                progress_km,
            } => {
                if *edge_index >= self.route.len() {
                    return [0.0, 0.0];
                }
                let edge_id = self.route[*edge_index] as usize;
                let edge_len = self.route_edge_lengths[*edge_index];
                let fraction = if edge_len > 0.0 {
                    (progress_km / edge_len).min(1.0)
                } else {
                    0.0
                };

                interpolate_along_edge(edge_geometries, edge_id, fraction)
            }
            AgentState::Arrived => {
                // Return last point of last edge
                if let Some(&last_eid) = self.route.last() {
                    let geom = &edge_geometries[last_eid as usize];
                    if let Some(last) = geom.last() {
                        return *last;
                    }
                }
                [0.0, 0.0]
            }
            AgentState::Idle => [0.0, 0.0],
        }
    }
}

/// Interpolate a position along an edge geometry at a given fraction (0..1).
fn interpolate_along_edge(
    edge_geometries: &[Vec<[f64; 2]>],
    edge_id: usize,
    fraction: f64,
) -> [f64; 2] {
    let coords = &edge_geometries[edge_id];
    if coords.is_empty() {
        return [0.0, 0.0];
    }
    if coords.len() == 1 || fraction <= 0.0 {
        return coords[0];
    }
    if fraction >= 1.0 {
        return *coords.last().unwrap();
    }

    // For simple 2-point edges, just lerp
    if coords.len() == 2 {
        let lng = coords[0][0] + fraction * (coords[1][0] - coords[0][0]);
        let lat = coords[0][1] + fraction * (coords[1][1] - coords[0][1]);
        return [lng, lat];
    }

    // For multi-point edges, find the right segment
    let mut total_len = 0.0_f64;
    let mut segment_lengths = Vec::with_capacity(coords.len() - 1);
    for i in 0..coords.len() - 1 {
        let d = approx_distance(coords[i], coords[i + 1]);
        segment_lengths.push(d);
        total_len += d;
    }

    let target = fraction * total_len;
    let mut accum = 0.0;
    for (i, &seg_len) in segment_lengths.iter().enumerate() {
        if accum + seg_len >= target {
            let seg_frac = if seg_len > 0.0 {
                (target - accum) / seg_len
            } else {
                0.0
            };
            let lng = coords[i][0] + seg_frac * (coords[i + 1][0] - coords[i][0]);
            let lat = coords[i][1] + seg_frac * (coords[i + 1][1] - coords[i][1]);
            return [lng, lat];
        }
        accum += seg_len;
    }

    *coords.last().unwrap()
}

/// Approximate distance between two [lng, lat] points (degree-based, fast).
fn approx_distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    let dlng = a[0] - b[0];
    let dlat = a[1] - b[1];
    (dlng * dlng + dlat * dlat).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_tick_basic() {
        let mut agent = Agent::new(0, 0, 1, vec![0], vec![1.0], false, false);
        // 60 km/h speed, 1 km edge => should take 60 seconds
        let edge_speeds = vec![60.0];
        let arrived = agent.tick(30.0, &edge_speeds);
        assert!(!arrived); // halfway
        assert!(agent.route_progress() > 0.4 && agent.route_progress() < 0.6);

        let arrived = agent.tick(31.0, &edge_speeds);
        assert!(arrived);
    }

    #[test]
    fn test_agent_position_interpolation() {
        let agent = Agent {
            id: 0,
            origin: 0,
            destination: 1,
            route: vec![0],
            route_edge_lengths: vec![1.0],
            state: AgentState::Traveling {
                edge_index: 0,
                progress_km: 0.5,
            },
            compliant: false,
            is_player: false,
        };

        let geometries = vec![vec![[78.0, 17.0], [79.0, 18.0]]];
        let pos = agent.position(&geometries);
        assert!((pos[0] - 78.5).abs() < 0.01);
        assert!((pos[1] - 17.5).abs() < 0.01);
    }
}
