use crate::{Connection, InputId, Node, NodeId, OutputId};
use std::{
    collections::{HashMap, LinkedList},
    fmt,
};

/// Processing graph consisting of nodes and connections.
pub struct Graph<N: Node> {
    /// Connections in graph.
    connections: Vec<Connection>,

    /// Internal counter for next node id.
    next_node_id: NodeId,

    /// Nodes in graph, indexed by unique id.
    nodes: HashMap<NodeId, N>,

    /// Node processing order (result of topologial sort).
    processing_order: LinkedList<NodeId>,
}
impl<N: Node> Graph<N> {
    /// Creates new empty graph.
    pub fn new() -> Self {
        Graph {
            connections: Vec::new(),
            next_node_id: NodeId(0),
            nodes: HashMap::new(),
            processing_order: LinkedList::new(),
        }
    }

    /// Adds a connection to the graph.
    pub fn add_connection(&mut self, connection: Connection) -> Result<Connection, GraphError> {
        // Validate connection and check whether input is free.
        let connection = self.validate_connection(connection)?;
        if self
            .connections
            .iter()
            .find(|c| c.target_node == connection.target_node && c.target_input == connection.target_input)
            .is_some()
        {
            return Err(GraphError::InputAlreadyConnected(connection.target_node, connection.target_input));
        }

        // Add connection, update processing order (check for undelayed cycles).
        self.connections.push(connection);
        match self.calc_processing_order() {
            Err(error) => {
                // Revert change (most likely an undelayed cycle was introduced).
                self.connections.pop();
                Err(error)
            }
            Ok(order) => {
                self.processing_order = order;
                Ok(connection)
            }
        }
    }

    /// Adds a node to the graph.
    pub fn add_node(&mut self, node: N) -> NodeId {
        let id = self.next_node_id;
        self.nodes.insert(id, node);
        self.processing_order = self.calc_processing_order().unwrap();
        self.next_node_id.0 += 1;
        id
    }

    /// Determines processing order (new topological sorting, can fail due to undelayed cycles).
    fn calc_processing_order(&self) -> Result<LinkedList<NodeId>, GraphError> {
        // Calculate in-degree of nodes.
        let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
        for &node in self.nodes.keys() {
            in_degree.insert(node, 0);
        }
        for connection in self.connections.iter() {
            // Nodes do not depend on nodes that introduce delay.
            if !self.get_node(connection.source_node).unwrap().delayed_processing() {
                in_degree.entry(connection.target_node).and_modify(|d| *d += 1);
            }
        }

        // Find nodes with in-degree 0.
        let mut queue: LinkedList<NodeId> = LinkedList::new();
        for (&node, &degree) in in_degree.iter() {
            if degree == 0 {
                queue.push_back(node);
            }
        }

        // Topological sort.
        let mut order: LinkedList<NodeId> = LinkedList::new();
        let mut delayed: LinkedList<NodeId> = LinkedList::new();
        while !queue.is_empty() {
            let node = queue.pop_front().unwrap();
            if !self.get_node(node).unwrap().delayed_processing() {
                // Reduce in-degree of connected nodes, add to queue once in-degree == 0.
                for connection in self.connections.iter() {
                    if connection.source_node == node {
                        in_degree.entry(connection.target_node).and_modify(|d| *d -= 1);
                        if *in_degree.get(&connection.target_node).unwrap() == 0 {
                            queue.push_back(connection.target_node);
                        }
                    }
                }
                order.push_back(node);
            } else {
                delayed.push_back(node);
            }
        }

        // Nodes that introduce delay are processed after all other nodes.
        while !delayed.is_empty() {
            order.push_back(delayed.pop_front().unwrap());
        }

        // Number of nodes in order won't match if an undelayed cycle exists.
        if order.len() != self.nodes.len() {
            return Err(GraphError::CycleWithoutDelay);
        }

        Ok(order)
    }

    /// Returns a node by id.
    pub fn get_node(&self, id: NodeId) -> Result<&N, GraphError> {
        self.nodes.get(&id).ok_or(GraphError::NodeNotExists(id))
    }

    /// Returns a mutable node by id.
    pub fn get_node_mut(&mut self, id: NodeId) -> Result<&mut N, GraphError> {
        self.nodes.get_mut(&id).ok_or(GraphError::NodeNotExists(id))
    }

    /// Returns iterator over nodes.
    pub fn iter_nodes(&self) -> impl Iterator<Item=(&NodeId, &N)> {
        self.nodes.iter()
    }

    /// Returns mutable iterator over nodes.
    pub fn iter_nodes_mut(&mut self) -> impl Iterator<Item=(&NodeId, &mut N)> {
        self.nodes.iter_mut()
    }

    /// Processes nodes in graph.
    pub fn process(&mut self) {
        for &node in self.processing_order.iter() {
            // Populate inputs.
            for connection in self.connections.iter() {
                if connection.target_node == node {
                    let value = self.nodes.get(&connection.source_node).unwrap().get_output(connection.source_output);
                    self.nodes.get_mut(&connection.target_node).unwrap().set_input(connection.target_input, value);
                }
            }

            // Process.
            self.nodes.get_mut(&node).unwrap().process();
        }
    }

    /// Removes a connection.
    pub fn remove_connection(&mut self, connection: Connection) -> Result<Connection, GraphError> {
        if self.connections.contains(&connection) {
            self.connections.retain(|&c| c != connection);
            self.processing_order = self.calc_processing_order().unwrap();
            Ok(connection)
        } else {
            Err(GraphError::ConnectionNotExists(connection))
        }
    }

    /// Removes a node by id.
    pub fn remove_node(&mut self, id: NodeId) -> Result<N, GraphError> {
        let node = self.nodes.remove(&id).ok_or(GraphError::NodeNotExists(id))?;
        self.connections = self.connections.iter().cloned().filter(|&c| self.validate_connection(c).is_ok()).collect();
        self.processing_order = self.calc_processing_order().unwrap();
        Ok(node)
    }

    /// Validates a connection (whether nodes and input/output exist).
    fn validate_connection(&self, connection: Connection) -> Result<Connection, GraphError> {
        let source = self.get_node(connection.source_node)?;
        let target = self.get_node(connection.target_node)?;
        if !source.list_outputs().contains(&connection.source_output) {
            return Err(GraphError::OutputNotExists(connection.source_node, connection.source_output));
        }
        if !target.list_inputs().contains(&connection.target_input) {
            return Err(GraphError::InputNotExists(connection.target_node, connection.target_input));
        }
        Ok(connection)
    }
}

/// Graph error type.
#[derive(PartialEq)]
pub enum GraphError {
    ConnectionNotExists(Connection),
    CycleWithoutDelay,
    InputAlreadyConnected(NodeId, InputId),
    InputNotExists(NodeId, InputId),
    NodeNotExists(NodeId),
    OutputNotExists(NodeId, OutputId),
}
impl fmt::Debug for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GraphError::ConnectionNotExists(connection) => write!(f, "{:?} does not exist in graph.", connection),
            GraphError::CycleWithoutDelay => write!(f, "Graph contains a cycle without delay."),
            GraphError::InputAlreadyConnected(node, input) => {
                write!(f, "Input with id {} on node with id {} is already connected.", input.0, node.0)
            }
            GraphError::InputNotExists(node, input) => {
                write!(f, "Input with id {} does not exist on node with id {}.", input.0, node.0)
            }
            GraphError::NodeNotExists(node) => write!(f, "Node with id {} does not exist in graph.", node.0),
            GraphError::OutputNotExists(node, output) => {
                write!(f, "Output with id {} does not exist on node with id {}.", output.0, node.0)
            }
        }
    }
}

/// Unit tests.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes;

    #[test]
    fn add_connection() {
        let mut graph: Graph<Box<dyn Node>> = Graph::new();
        let node0 = graph.add_node(Box::from(nodes::Variable::new(1.0)));
        let node1 = graph.add_node(Box::from(nodes::Variable::new(2.0)));
        assert_eq!(graph.connections.len(), 0);

        graph.add_connection(Connection::new(node0, OutputId(0), node1, InputId(0))).unwrap();
        assert_eq!(graph.connections.len(), 1);

        // Invalid connections.
        assert_eq!(
            graph.add_connection(Connection::new(node0, OutputId(0), node1, InputId(0))),
            Err(GraphError::InputAlreadyConnected(node1, InputId(0)))
        );
        assert_eq!(
            graph.add_connection(Connection::new(node1, OutputId(0), node0, InputId(0))),
            Err(GraphError::CycleWithoutDelay)
        );
        assert_eq!(
            graph.add_connection(Connection::new(node0, OutputId(0), node1, InputId(1))),
            Err(GraphError::InputNotExists(node1, InputId(1)))
        );
        assert_eq!(
            graph.add_connection(Connection::new(NodeId(2), OutputId(0), node1, InputId(0))),
            Err(GraphError::NodeNotExists(NodeId(2)))
        );
        assert_eq!(
            graph.add_connection(Connection::new(node0, OutputId(1), node1, InputId(0))),
            Err(GraphError::OutputNotExists(node0, OutputId(1)))
        );
        assert_eq!(graph.connections.len(), 1);
    }

    #[test]
    fn add_node() {
        let mut graph: Graph<Box<dyn Node>> = Graph::new();
        assert_eq!(graph.nodes.len(), 0);

        let node0 = graph.add_node(Box::from(nodes::Variable::new(1.0)));
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(node0, NodeId(0));

        let node1 = graph.add_node(Box::from(nodes::Variable::new(2.0)));
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(node1, NodeId(1));
    }

    #[test]
    fn processing_order() {
        let mut graph: Graph<Box<dyn Node>> = Graph::new();
        let var0 = graph.add_node(Box::from(nodes::Variable::new(1.0)));
        let var1 = graph.add_node(Box::from(nodes::Variable::new(2.0)));
        graph.add_connection(Connection::new(var0, OutputId(0), var1, InputId(0))).unwrap();
        assert_eq!(graph.processing_order.iter().cloned().collect::<Vec<NodeId>>(), vec![var0, var1]);

        let add2 = graph.add_node(Box::from(nodes::Addition::new()));
        graph.add_connection(Connection::new(var0, OutputId(0), add2, InputId(0))).unwrap();
        graph.add_connection(Connection::new(var1, OutputId(0), add2, InputId(1))).unwrap();
        assert_eq!(graph.processing_order.iter().cloned().collect::<Vec<NodeId>>(), vec![var0, var1, add2]);

        let var3 = graph.add_node(Box::from(nodes::Variable::new(3.0)));
        graph.add_connection(Connection::new(var3, OutputId(0), var0, InputId(0))).unwrap();
        assert_eq!(graph.processing_order.iter().cloned().collect::<Vec<NodeId>>(), vec![var3, var0, var1, add2]);

        let delay4 = graph.add_node(Box::from(nodes::Delay::new()));
        let add5 = graph.add_node(Box::from(nodes::Addition::new()));
        graph.add_connection(Connection::new(add5, OutputId(0), delay4, InputId(0))).unwrap();
        graph.add_connection(Connection::new(delay4, OutputId(0), add5, InputId(0))).unwrap();
        graph.add_connection(Connection::new(add2, OutputId(0), add5, InputId(1))).unwrap();
        assert_eq!(
            graph.processing_order.iter().cloned().collect::<Vec<NodeId>>(),
            vec![var3, var0, var1, add2, add5, delay4]
        );
    }

    #[test]
    fn get_node() {
        let mut graph: Graph<Box<dyn Node>> = Graph::new();
        assert_eq!(graph.get_node(NodeId(0)).err(), Some(GraphError::NodeNotExists(NodeId(0))));
        assert_eq!(graph.get_node_mut(NodeId(0)).err(), Some(GraphError::NodeNotExists(NodeId(0))));
        graph.add_node(Box::from(nodes::Variable::new(1.0)));
        assert_eq!(graph.get_node(NodeId(0)).map(|n| n.get_output(OutputId(0))), Ok(1.0));
        assert_eq!(graph.get_node_mut(NodeId(0)).map(|n| n.get_output(OutputId(0))), Ok(1.0));
    }

    #[test]
    fn iter_node() {
        let mut graph: Graph<Box<dyn Node>> = Graph::new();
        let var0 = graph.add_node(Box::from(nodes::Variable::new(1.0)));
        let var1 = graph.add_node(Box::from(nodes::Variable::new(2.0)));
        let var2 = graph.add_node(Box::from(nodes::Variable::new(3.0)));
        assert!(graph.iter_nodes().find(|(&id, _)| id == var0).is_some());
        assert!(graph.iter_nodes().find(|(&id, _)| id == var1).is_some());
        assert!(graph.iter_nodes_mut().find(|(&id, _)| id == var2).is_some());
    }

    #[test]
    fn process() {
        let mut graph: Graph<Box<dyn Node>> = Graph::new();
        let var0 = graph.add_node(Box::from(nodes::Variable::new(1.0)));
        let add1 = graph.add_node(Box::from(nodes::Addition::new()));
        let del2 = graph.add_node(Box::from(nodes::Delay::new()));
        graph.add_connection(Connection::new(var0, OutputId(0), add1, InputId(0))).unwrap();
        graph.add_connection(Connection::new(add1, OutputId(0), del2, InputId(0))).unwrap();
        graph.add_connection(Connection::new(del2, OutputId(0), add1, InputId(1))).unwrap();
        assert_eq!(graph.processing_order.iter().cloned().collect::<Vec<NodeId>>(), vec![var0, add1, del2]);

        assert_eq!(graph.get_node(add1).unwrap().get_output(OutputId(0)), 0.0);
        graph.process();
        assert_eq!(graph.get_node(add1).unwrap().get_output(OutputId(0)), 1.0);
        graph.process();
        assert_eq!(graph.get_node(add1).unwrap().get_output(OutputId(0)), 2.0);
        graph.process();
        assert_eq!(graph.get_node(add1).unwrap().get_output(OutputId(0)), 3.0);
    }

    #[test]
    fn remove_connection() {
        let mut graph: Graph<Box<dyn Node>> = Graph::new();
        let node0 = graph.add_node(Box::from(nodes::Variable::new(1.0)));
        let node1 = graph.add_node(Box::from(nodes::Variable::new(2.0)));
        let node2 = graph.add_node(Box::from(nodes::Variable::new(3.0)));
        let conn0 = Connection::new(node2, OutputId(0), node1, InputId(0));
        let conn1 = Connection::new(node1, OutputId(0), node0, InputId(0));
        graph.add_connection(conn0).unwrap();
        graph.add_connection(conn1).unwrap();
        assert_eq!(graph.connections.len(), 2);

        assert_eq!(graph.remove_connection(conn1), Ok(conn1));
        assert_eq!(graph.connections.len(), 1);

        assert_eq!(graph.remove_connection(conn1), Err(GraphError::ConnectionNotExists(conn1)));
    }

    #[test]
    fn remove_node() {
        let mut graph: Graph<Box<dyn Node>> = Graph::new();
        let node0 = graph.add_node(Box::from(nodes::Variable::new(1.0)));
        let node1 = graph.add_node(Box::from(nodes::Variable::new(2.0)));
        let node2 = graph.add_node(Box::from(nodes::Variable::new(3.0)));
        assert_eq!(graph.nodes.len(), 3);
        let conn0 = Connection::new(node2, OutputId(0), node1, InputId(0));
        let conn1 = Connection::new(node1, OutputId(0), node0, InputId(0));
        graph.add_connection(conn0).unwrap();
        graph.add_connection(conn1).unwrap();
        assert_eq!(graph.connections.len(), 2);

        assert_eq!(graph.remove_node(node1).map(|n| n.get_output(OutputId(0))), Ok(2.0));
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.connections.len(), 0);

        assert_eq!(graph.remove_node(node1).err(), Some(GraphError::NodeNotExists(node1)));
    }
}
