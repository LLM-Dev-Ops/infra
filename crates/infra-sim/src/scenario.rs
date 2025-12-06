//! Test scenario utilities.

use infra_errors::InfraResult;
use std::time::Duration;

/// A step in a scenario
#[derive(Debug, Clone)]
pub struct Step {
    /// Step name
    pub name: String,
    /// Step description
    pub description: Option<String>,
    /// Delay before executing
    pub delay: Option<Duration>,
}

impl Step {
    /// Create a new step
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            delay: None,
        }
    }

    /// Add a description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a delay
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }
}

/// Test scenario
pub struct Scenario {
    /// Scenario name
    name: String,
    /// Steps in the scenario
    steps: Vec<Step>,
    /// Current step index
    current: usize,
}

impl Scenario {
    /// Create a new scenario
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            steps: Vec::new(),
            current: 0,
        }
    }

    /// Get the scenario name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get all steps
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// Get the current step
    pub fn current_step(&self) -> Option<&Step> {
        self.steps.get(self.current)
    }

    /// Advance to the next step
    pub fn advance(&mut self) -> bool {
        if self.current < self.steps.len() {
            self.current += 1;
            true
        } else {
            false
        }
    }

    /// Check if the scenario is complete
    pub fn is_complete(&self) -> bool {
        self.current >= self.steps.len()
    }

    /// Reset the scenario
    pub fn reset(&mut self) {
        self.current = 0;
    }
}

/// Scenario builder
pub struct ScenarioBuilder {
    scenario: Scenario,
}

impl ScenarioBuilder {
    /// Create a new builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            scenario: Scenario::new(name),
        }
    }

    /// Add a step
    pub fn step(mut self, step: Step) -> Self {
        self.scenario.steps.push(step);
        self
    }

    /// Add a simple step by name
    pub fn add_step(self, name: impl Into<String>) -> Self {
        self.step(Step::new(name))
    }

    /// Build the scenario
    pub fn build(self) -> Scenario {
        self.scenario
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario() {
        let mut scenario = ScenarioBuilder::new("test-scenario")
            .add_step("setup")
            .add_step("execute")
            .add_step("verify")
            .add_step("cleanup")
            .build();

        assert_eq!(scenario.name(), "test-scenario");
        assert_eq!(scenario.steps().len(), 4);

        assert!(!scenario.is_complete());
        assert_eq!(scenario.current_step().unwrap().name, "setup");

        scenario.advance();
        assert_eq!(scenario.current_step().unwrap().name, "execute");

        scenario.advance();
        scenario.advance();
        scenario.advance();
        assert!(scenario.is_complete());
    }
}
