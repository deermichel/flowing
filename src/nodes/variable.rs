use crate::{InputId, Node, OutputId};

/// Node that holds a variable value.
pub struct Variable {
    value: f64,
}
impl Variable {
    /// Creates new variable node with initial value.
    pub fn new(value: f64) -> Self {
        Variable { value }
    }
}
impl Node for Variable {
    fn delayed_processing(&self) -> bool {
        false
    }

    fn get_output(&self, id: OutputId) -> f64 {
        match id.0 {
            0 => self.value,
            _ => panic!("Output with id {} does not exist.", id.0),
        }
    }

    fn list_inputs(&self) -> &[InputId] {
        // 0 -> value.
        &[InputId(0)]
    }

    fn list_outputs(&self) -> &[OutputId] {
        // 0 -> value.
        &[OutputId(0)]
    }

    fn process(&mut self) {
        // Passthrough noop.
    }

    fn set_input(&mut self, id: InputId, value: f64) {
        match id.0 {
            0 => self.value = value,
            _ => panic!("Input with id {} does not exist.", id.0),
        }
    }
}

/// Unit tests.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holds_value() {
        let mut var = Variable::new(42.0);
        assert_eq!(var.get_output(OutputId(0)), 42.0);

        var.set_input(InputId(0), 2.0);
        assert_eq!(var.get_output(OutputId(0)), 2.0);
    }
}
