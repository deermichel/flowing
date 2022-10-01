use crate::{InputId, Node, OutputId};

/// Node that delays the input by one processing cycle.
pub struct Delay {
    value: (f64, f64),
}
impl Delay {
    /// Creates new delay node.
    pub fn new() -> Self {
        Delay { value: (0.0, 0.0) }
    }
}
impl Node for Delay {
    fn delayed_processing(&self) -> bool {
        true
    }

    fn get_output(&self, id: OutputId) -> f64 {
        match id.0 {
            0 => self.value.1,
            _ => panic!("Output with id {} does not exist.", id.0),
        }
    }

    fn list_inputs(&self) -> &[InputId] {
        // 0 -> value.0 (input).
        &[InputId(0)]
    }

    fn list_outputs(&self) -> &[OutputId] {
        // 0 -> value.1 (delayed output).
        &[OutputId(0)]
    }

    fn process(&mut self) {
        // Since delay nodes are processed last, output changes will be visible only in the next processing cycle.
        self.value.1 = self.value.0;
    }

    fn set_input(&mut self, id: InputId, value: f64) {
        match id.0 {
            0 => self.value.0 = value,
            _ => panic!("Input with id {} does not exist.", id.0),
        }
    }
}

/// Unit tests.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delays_value() {
        let mut del = Delay::new();
        del.set_input(InputId(0), 2.0);
        assert_eq!(del.get_output(OutputId(0)), 0.0);

        del.process();
        assert_eq!(del.get_output(OutputId(0)), 2.0);

        del.process();
        assert_eq!(del.get_output(OutputId(0)), 2.0);
    }
}
