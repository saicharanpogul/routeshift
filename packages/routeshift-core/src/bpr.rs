/// Bureau of Public Roads (BPR) latency function.
///
/// Computes travel time on an edge as a function of flow:
///   l(x) = free_flow_time * (1 + alpha * (x / capacity)^beta)
///
/// Standard parameters: alpha = 0.15, beta = 4.0

const ALPHA: f64 = 0.15;
const BETA: f64 = 4.0;

/// Compute travel time on an edge given current flow.
///
/// BPR formula: t_e * (1 + 0.15 * (x / c)^4)
#[inline]
pub fn travel_time(free_flow_time: f64, flow: f64, capacity: f64) -> f64 {
    if capacity <= 0.0 {
        return f64::MAX;
    }
    let ratio = flow / capacity;
    free_flow_time * (1.0 + ALPHA * ratio.powi(BETA as i32))
}

/// Compute marginal cost for System Optimal assignment.
///
/// The marginal cost is d(x * l(x))/dx:
///   t_e * (1 + alpha * (beta + 1) * (x / c)^beta)
///   = t_e * (1 + 0.15 * 5 * (x / c)^4)
///   = t_e * (1 + 0.75 * (x / c)^4)
#[inline]
pub fn marginal_cost(free_flow_time: f64, flow: f64, capacity: f64) -> f64 {
    if capacity <= 0.0 {
        return f64::MAX;
    }
    let ratio = flow / capacity;
    free_flow_time * (1.0 + ALPHA * (BETA + 1.0) * ratio.powi(BETA as i32))
}

/// Compute the Beckmann objective function integral for an edge.
///
/// Integral of l(x) dx from 0 to flow:
///   free_flow_time * (flow + alpha * capacity / (beta + 1) * (flow / capacity)^(beta + 1))
#[inline]
pub fn beckmann_integral(free_flow_time: f64, flow: f64, capacity: f64) -> f64 {
    if capacity <= 0.0 {
        return 0.0;
    }
    let ratio = flow / capacity;
    free_flow_time * (flow + ALPHA * capacity / (BETA + 1.0) * ratio.powi((BETA + 1.0) as i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_travel_time_zero_flow() {
        let tt = travel_time(10.0, 0.0, 100.0);
        assert!((tt - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_travel_time_at_capacity() {
        // At flow = capacity: t * (1 + 0.15 * 1^4) = t * 1.15
        let tt = travel_time(10.0, 100.0, 100.0);
        assert!((tt - 11.5).abs() < 1e-10);
    }

    #[test]
    fn test_travel_time_over_capacity() {
        // At flow = 2*capacity: t * (1 + 0.15 * 16) = t * 3.4
        let tt = travel_time(10.0, 200.0, 100.0);
        assert!((tt - 34.0).abs() < 1e-10);
    }

    #[test]
    fn test_marginal_cost_zero_flow() {
        let mc = marginal_cost(10.0, 0.0, 100.0);
        assert!((mc - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_marginal_cost_at_capacity() {
        // At flow = capacity: t * (1 + 0.75 * 1^4) = t * 1.75
        let mc = marginal_cost(10.0, 100.0, 100.0);
        assert!((mc - 17.5).abs() < 1e-10);
    }

    #[test]
    fn test_beckmann_zero_flow() {
        let bi = beckmann_integral(10.0, 0.0, 100.0);
        assert!((bi - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_zero_capacity_returns_max() {
        let tt = travel_time(10.0, 50.0, 0.0);
        assert_eq!(tt, f64::MAX);
    }
}
