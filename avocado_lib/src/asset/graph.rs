use crate::incl::*;

pub type AssetGraphIn = HashMap<&'static str, AssetGraphOut>;
pub type AssetGraphOut = Vec<HandleDyn>;
pub type AssetGraphResult = Result<AssetGraphOut, anyhow::Error>;

#[derive(Resource)]
pub struct AssetGraph {
    nodes: Vec<AssetGraphNode>,
    data: AssetGraphIn,

    ran: bool,
    to_update: Vec<(usize, AssetGraphOut)>,
    next_update: HashSet<usize>,
}

impl AssetGraph {
    pub fn update_sys(world: &mut World) {
        if world.contains_resource::<AssetGraph>() {
            let done = Rc::new(RwLock::new(false));
            let error = Rc::new(RwLock::new(None));

            {
                let done = Rc::clone(&done);
                let error = Rc::clone(&error);

                world.resource_scope(|world, mut graph: Mut<AssetGraph>| match graph.update(world) {
                    Ok(true) => *done.write() = true,
                    Ok(false) => {},
                    Err(msg) => *error.write() = Some(msg),
                });
            }

            if let Some(ref msg) = *error.read() {
                world.remove_resource::<AssetGraph>();
                world.send_event(ExitEvent::error(msg.clone()));
            } else if *done.read() {
                world.remove_resource::<AssetGraph>();
                world.send_event(AssetGraphDoneEvent);
            }; // This semicolon is necessary.
        }
    }

    fn new(nodes: Vec<AssetGraphNode>) -> Self {
        Self {
            nodes,
            data: AssetGraphIn::default(),

            ran: false,
            to_update: vec![],
            next_update: HashSet::default(),
        }
    }

    pub fn update(&mut self, world: &mut World) -> Result<bool, String> {
        if self.to_update.is_empty() {
            if self.ran {
                return Ok(true);
            } else {
                self.ran = true;
                for i in 0..self.nodes.len() {
                    let node = &mut self.nodes[i];
                    if node.parents.is_empty() {
                        node.system.initialize(world);
                        self.to_update.push((i, node.system
                            .run(AssetGraphIn::default(), world)
                            .map_err(|e| e.to_string())?
                        ));

                        node.system.apply_buffers(world);
                    }
                }
            }
        }

        let server = world.resource::<AssetServer>();
        let mut i = 0;
        while i < self.to_update.len() {
            let (index, handles) = &mut self.to_update[i];
            match server.group_state_dyn(handles.iter())  {
                AssetState::Loading => {
                    i += 1;
                    continue;
                },
                AssetState::Loaded => {
                    for child_index in &self.nodes[*index].children {
                        self.next_update.insert(*child_index);
                    }

                    self.data.insert(self.nodes[*index].label, {
                        let mut vec = Vec::with_capacity(handles.len());
                        vec.append(handles);
                        vec
                    });

                    self.to_update.remove(i);
                },
                AssetState::Errored(msg) => return Err(msg),
                AssetState::Unloaded => return Err(format!("Asset handles from {} are unloaded", &self.nodes[*index])),
            }
        }

        for next_index in self.next_update.drain_filter(|index| {
            let mut qualified = true;

            let child = &self.nodes[*index];
            for parent_index in &child.parents {
                let parent = &self.nodes[*parent_index];
                if !self.data.contains_key(&parent.label) {
                    qualified = false;
                    break;
                }
            }

            qualified
        }).collect::<Vec<_>>() {
            let mut data = AssetGraphIn::default();
            for parent_index in &self.nodes[next_index].parents {
                let parent = &self.nodes[*parent_index];
                data.insert(parent.label, self.data[&parent.label].clone());
            }

            let next = &mut self.nodes[next_index];
            next.system.initialize(world);
            self.to_update.push((next_index, next.system
                .run(data, world)
                .map_err(|e| e.to_string())?
            ));

            next.system.apply_buffers(world);
        }

        Ok(false)
    }
}

#[derive(Display)]
#[display(fmt = "{:?}", label)]
pub struct AssetGraphNode {
    label: &'static str,
    system: BoxedSystem<AssetGraphIn, AssetGraphResult>,
    
    parents: Vec<usize>,
    children: Vec<usize>,
}

#[derive(Default)]
pub struct AssetGraphBuilder {
    nodes: Vec<AssetGraphNode>,
    labels: HashMap<&'static str, usize>,
}

impl AssetGraphBuilder {
    pub fn build(self) -> AssetGraph {
        AssetGraph::new(self.nodes)
    }

    pub fn node<Param>(
        &mut self,
        label: &'static str, sys: impl IntoSystem<AssetGraphIn, AssetGraphResult, Param>
    ) {
        if self.labels.insert(label, self.nodes.len()).is_some() {
            panic!("Duplicate label: {:?}", label);
        }

        self.nodes.push(AssetGraphNode {
            label,
            system: Box::new(IntoSystem::into_system(sys)),
            
            parents: vec![],
            children: vec![],
        });
    }

    pub fn edge(&mut self, parent: &'static str, child: &'static str) {
        let parent = self.labels[parent];
        let child = self.labels[child];

        if self.nodes[parent].children.contains(&child) {
            return;
        }

        let mut queue = VecDeque::<usize>::new();
        let mut iterated = HashSet::default();
        queue.extend(&self.nodes[child].children);

        loop {
            let c = match queue.pop_front() {
                Some(c) => c,
                None => break,
            };

            if c == parent {
                panic!("Asset node {} circles with itself", &self.nodes[child]);
            } else if iterated.insert(c) {
                queue.extend(&self.nodes[c].children);
            }
        }

        self.nodes[child].parents.push(parent);
        self.nodes[parent].children.push(child);
    }
}
