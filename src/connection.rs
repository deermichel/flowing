use crate::{InputId, NodeId, OutputId};

/// Graph edge between source node output and target node input.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Connection {
    pub source_node: NodeId,
    pub source_output: OutputId,
    pub target_input: InputId,
    pub target_node: NodeId,
}
impl Connection {
    /// Creates new connection.
    pub fn new(source_node: NodeId, source_output: OutputId, target_node: NodeId, target_input: InputId) -> Self {
        Connection { source_node, source_output, target_input, target_node }
    }
}
