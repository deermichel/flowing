use crate::{InputId, Node, OutputId};

/// Node that adds two values.
pub struct Addition {
    summands: (f64, f64),
    sum: f64,
}
impl Addition {
    /// Creates new addition node.
    pub fn new() -> Self {
        Addition { summands: (0.0, 0.0), sum: 0.0 }
    }
}
impl Node for Addition {
    fn delayed_processing(&self) -> bool {
        false
    }

    fn get_output(&self, id: OutputId) -> f64 {
        match id.0 {
            0 => self.sum,
            _ => panic!("Output with id {} does not exist.", id.0),
        }
    }

    fn list_inputs(&self) -> &[InputId] {
        // 0 -> 1st summand.
        // 1 -> 2nd summand.
        &[InputId(0), InputId(1)]
    }

    fn list_outputs(&self) -> &[OutputId] {
        // 0 -> sum.
        &[OutputId(0)]
    }

    fn process(&mut self) {
        self.sum = self.summands.0 + self.summands.1;
    }

    fn set_input(&mut self, id: InputId, value: f64) {
        match id.0 {
            0 => self.summands.0 = value,
            1 => self.summands.1 = value,
            _ => panic!("Input with id {} does not exist.", id.0),
        }
    }
}

/// Unit tests.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_values() {
        let mut add = Addition::new();
        assert_eq!(add.get_output(OutputId(0)), 0.0);

        add.set_input(InputId(0), 2.0);
        add.set_input(InputId(1), 32.0);
        add.process();
        assert_eq!(add.get_output(OutputId(0)), 34.0);

        add.set_input(InputId(0), 4.0);
        add.process();
        assert_eq!(add.get_output(OutputId(0)), 36.0);
    }
}
