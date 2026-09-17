pub struct DeliberationBridge;

impl DeliberationBridge {
    /// Evaluates prompt intent through biological connectome spiking simulation to compute deliberation confidence
    pub fn deliberate_intent(
        prompt: &str,
    ) -> (f32, u64, Option<connectome::DecisionResult>) {
        let trimmed = prompt.trim();
        let is_action = trimmed.to_lowercase().contains("create")
            || trimmed.to_lowercase().contains("fix")
            || trimmed.to_lowercase().contains("refactor")
            || trimmed.to_lowercase().contains("add")
            || trimmed.to_lowercase().contains("build");

        let (ev_a, ev_b) = if is_action {
            (0.85, 0.15)
        } else {
            (0.3, 0.7)
        };

        let sim = connectome::ConnectomeRNN::load_default();
        let result = sim.deliberate(ev_a, ev_b, 30);
        let confidence = result.confidence;
        let dt = result.reaction_time_dt as u64;

        (confidence, dt, Some(result))
    }
}
