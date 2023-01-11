#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
    sync::{Arc, RwLock},
    thread,
};

use eframe::egui;
use egui::Context;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum Node {
    TextNode { text: String },
}

#[derive(Serialize, Deserialize)]
enum EditCommand {
    AppendChild {
        parent_id: u32,
        object_id: u32,
        node: Node,
    },
}

#[derive(Serialize, Deserialize)]
struct Transaction {
    client_id: String,

    edits: Vec<EditCommand>,
}

struct SceneNodeImpl {
    id: u32,
    node: Option<Node>,
    children: Vec<SceneNode>,
}

#[derive(Clone)]
struct SceneNode(Arc<RwLock<SceneNodeImpl>>);
impl SceneNode {
    fn new(id: u32) -> SceneNode {
        SceneNode(Arc::new(RwLock::new(SceneNodeImpl {
            id,
            node: None,
            children: vec![],
        })))
    }

    fn draw(&self, ctx: &egui::Context, ui: &mut egui::Ui) -> () {
        let val = self.0.read().unwrap();

        match &val.node {
            Some(Node::TextNode { text }) => {
                ui.label(text.clone());
            }
            None => {}
        }

        for child in &val.children {
            child.draw(ctx, ui);
        }
    }

    fn set_node(&self, node: Node) {
        self.0.write().unwrap().node = Some(node)
    }

    fn append(&self, new_child: SceneNode) {
        self.0.write().unwrap().children.push(new_child);
    }

    fn id(&self) -> u32 {
        self.0.read().unwrap().id
    }

    fn get_child(&self, id: u32) -> Option<SceneNode> {
        if self.id() == id {
            Some(self.clone())
        } else {
            for child in &self.0.read().unwrap().children {
                match child.get_child(id) {
                    Some(child) => return Some(child),
                    None => continue,
                }
            }
            None
        }
    }
}

const ROOT_ID: u32 = 0xff_ff_ff_ff;

struct Scene {
    root: SceneNode,
}

impl Scene {
    fn new() -> Scene {
        Self {
            root: SceneNode::new(ROOT_ID),
        }
    }

    fn draw(&self, ctx: &egui::Context, ui: &mut egui::Ui) -> () {
        self.root.draw(ctx, ui);
    }

    fn get_child(&self, id: u32) -> Option<SceneNode> {
        self.root.get_child(id)
    }
}

struct SocketListener {
    scene: Arc<Scene>,
    ctx: Context,
}

impl SocketListener {
    fn new(scene: Arc<Scene>, ctx: Context) -> Self {
        Self { scene, ctx }
    }

    pub(crate) fn handle_transaction(&self, tx: Transaction) {
        self.ctx.request_repaint();

        for edit in tx.edits {
            match edit {
                EditCommand::AppendChild {
                    parent_id,
                    object_id,
                    node,
                } => {
                    let parent = self.scene.get_child(parent_id);
                    if let Some(parent) = parent {
                        let new_child = SceneNode::new(object_id);
                        new_child.set_node(node);
                        parent.append(new_child);
                    }
                }
            }
        }
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
    scene: Arc<Scene>,
    listening: bool,
}
impl Application {
    fn new(scene: Arc<Scene>) -> Application {
        Application {
            scene: scene,
            listening: false,
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.listening {
            let scene_clone = self.scene.clone();
            let ctx_clone = ctx.clone();

            thread::spawn(move || {
                SocketListener::new(scene_clone, ctx_clone).listen();
            });

            self.listening = true;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.scene.draw(ctx, ui);
        });
    }
}

fn main() {
    let scene = Arc::new(Scene::new());
    let app_scene = scene.clone();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(Application::new(app_scene))),
    )
}
