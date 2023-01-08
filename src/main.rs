use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

use eframe::egui;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
enum SceneNode {
    EmptyNode {},
    TextNode { text: String },
}

impl SceneNode {
    fn draw(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        match self {
            SceneNode::EmptyNode {} => {}
            SceneNode::TextNode { text } => {
                ui.label(text.clone());
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum EditCommand {
    AppendChild {
        parent_id: u32,
        object_id: u32,
        node: SceneNode,
    },
    ReplaceNode {
        object_id: u32,
        node: SceneNode,
    },
    CleanupClient {},
    DeleteObject {
        object_id: u32,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct Transaction {
    client_id: String,

    edits: Vec<EditCommand>,
}

struct SceneObject {
    id: u32,
    node: Option<SceneNode>,
    children: Vec<SceneObjectPtr>,
}

type SceneObjectPtr = Arc<Mutex<SceneObject>>;

impl SceneObject {
    fn new(id: u32) -> SceneObjectPtr {
        Arc::new(Mutex::new(Self {
            id,
            node: None,
            children: vec![],
        }))
    }

    fn set_node(&mut self, node: Option<SceneNode>) {
        self.node = node;
    }

    fn append(&mut self, child: SceneObjectPtr) {
        self.children.push(child);
    }

    fn draw(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if let Some(node) = &mut self.node {
            node.draw(ctx, ui);

            for child in self.children.iter_mut() {
                child.lock().unwrap().draw(ctx, ui);
            }
        } else {
            for child in self.children.iter_mut() {
                child.lock().unwrap().draw(ctx, ui);
            }
        }
    }
}

const ROOT_ID: u32 = 0xff_ff_ff_ff;

struct UIScene {
    root: SceneObjectPtr,
    node_map: HashMap<u32, SceneObjectPtr>,
}

impl UIScene {
    fn new() -> Self {
        let root = SceneObject::new(ROOT_ID);
        let mut node_map = HashMap::new();
        let node_entry = root.clone();
        node_map.insert(ROOT_ID, node_entry);
        Self { root, node_map }
    }

    fn add(&mut self, node: SceneObjectPtr) {
        let id = node.lock().unwrap().id;
        self.node_map.entry(id).and_modify(|e| *e = node);
    }

    fn get_object_mut(&mut self, id: u32) -> Option<SceneObjectPtr> {
        self.node_map.get(&id).cloned()
    }

    fn draw(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        self.root.lock().unwrap().draw(ctx, ui);
    }

    fn process(&mut self, tx: Transaction) {
        println!("process: {:?}", tx);

        for edit in tx.edits {
            match edit {
                EditCommand::AppendChild {
                    parent_id,
                    object_id,
                    node,
                } => {
                    let parent = self.get_object_mut(parent_id);
                    if let Some(parent) = parent {
                        let new_obj = SceneObject::new(object_id);
                        new_obj.lock().unwrap().set_node(Some(node));
                        let new_obj_clone = new_obj.clone();
                        self.add(new_obj);
                        parent.lock().unwrap().append(new_obj_clone);
                    } else {
                        println!("parent {} not found.", parent_id);
                    }
                }
                EditCommand::ReplaceNode { object_id, node } => {
                    let obj = self.get_object_mut(object_id);
                    if let Some(obj) = obj {
                        obj.lock().unwrap().set_node(Some(node));
                    } else {
                        println!("object {} not found.", object_id);
                    }
                }
                EditCommand::CleanupClient {} => todo!(),
                EditCommand::DeleteObject { object_id } => todo!(),
            }
        }
    }
}

struct SocketListener {
    scene: Arc<Mutex<UIScene>>,
}

impl SocketListener {
    fn new(socket: Arc<Mutex<UIScene>>) -> Self {
        Self { scene: socket }
    }

    fn handle_transaction(&self, tx: Transaction) {
        self.scene.lock().unwrap().process(tx);
    }

    fn listen(&self) {
        println!("listening on ws://127.0.0.1:3012/");
        ws::listen("127.0.0.1:3012", |out| {
            move |msg: ws::Message| {
                if let Ok(text) = msg.into_text() {
                    match serde_json::from_str::<Transaction>(&text) {
                        Ok(tx) => {
                            self.handle_transaction(tx);
                            Ok(())
                        }
                        Err(err) => {
                            println!("error deserializing: {:?}", err);
                            Ok(())
                        }
                    }
                } else {
                    Ok(())
                }
            }
        })
        .unwrap();
    }
}

struct Application {
    scene: Arc<Mutex<UIScene>>,
}

impl Application {
    fn new(socket: Arc<Mutex<UIScene>>) -> Self {
        Self { scene: socket }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        println!("render");
        // Render the current scene.
        egui::CentralPanel::default().show(ctx, |ui| {
            self.scene.lock().unwrap().draw(ctx, ui);
        });
    }
}

fn main() {
    let scene = Arc::new(Mutex::new(UIScene::new()));
    let scene_socket = Arc::clone(&scene);

    thread::spawn(move || {
        let listener = SocketListener::new(scene_socket);
        listener.listen();
    });

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(Application::new(scene))),
    )
}
