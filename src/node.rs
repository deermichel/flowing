/// Identifier for input (unique in node).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InputId(pub u32);

/// Identifier for node (unique in graph).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId(pub u32);

/// Identifier for output (unique in node).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct OutputId(pub u32);

/// Abstract processing node with inputs and outputs.
pub trait Node {
    /// Returns whether node introduces processing delay.
    fn delayed_processing(&self) -> bool;

    /// Returns output value.
    fn get_output(&self, id: OutputId) -> f64;

    /// Returns all available inputs.
    fn list_inputs(&self) -> &[InputId];

    /// Returns all available outputs.
    fn list_outputs(&self) -> &[OutputId];

    /// Processes values.
    fn process(&mut self);

    /// Sets input value.
    fn set_input(&mut self, id: InputId, value: f64);
}
