#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
    sync::{Arc, Mutex, RwLock},
    thread,
};

use eframe::egui;
use egui::Context;
use serde_derive::{Deserialize, Serialize};
use ws::Sender;

#[derive(Serialize, Deserialize)]
enum Node {
    Label { text: String, heading: Option<bool> },
    TextInput { text: String, on_changed: String },
    Button { text: String, on_clicked: String },
    LeftToRightLayout {},
    Window { title: String },
}

#[derive(Serialize, Deserialize)]
enum EditCommand {
    AppendChild {
        parent_id: u32,
        object_id: u32,
        node: Node,
    },
    ReplaceNode {
        object_id: u32,
        node: Node,
    },
}

#[derive(Serialize, Deserialize)]
struct Transaction {
    client_id: String,

    edits: Vec<EditCommand>,
}

#[derive(Serialize, Deserialize)]
enum Event {
    TextChanged { id: String, text: String },
    Clicked { id: String },
}

struct ClientImpl {
    senders: Vec<Sender>,
}

#[derive(Clone)]
struct Client(Arc<Mutex<ClientImpl>>);
impl Client {
    fn send_event(&self, event: Event) {
        let val = self.0.lock().unwrap();

        let msg = serde_json::to_string(&event).unwrap();

        for sender in &val.senders {
            sender.broadcast(msg.clone()).unwrap();
        }
    }

    fn add_sender(&self, sender: Sender) {
        self.0.lock().unwrap().senders.push(sender);
    }

    fn new() -> Client {
        Client(Arc::new(Mutex::new(ClientImpl { senders: vec![] })))
    }
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

    fn draw(&self, client: &Client, ctx: &egui::Context, ui: &mut egui::Ui) -> () {
        let mut val = self.0.write().unwrap();

        match &mut val.node {
            Some(Node::Label { text, heading }) => {
                if let Some(true) = heading {
                    ui.heading(text.clone());
                } else {
                    ui.label(text.clone());
                }
            }
            Some(Node::TextInput { text, on_changed }) => {
                let resp = ui.text_edit_singleline(text);

                if resp.changed() {
                    client.send_event(Event::TextChanged {
                        id: on_changed.to_string(),
                        text: text.to_string(),
                    });
                }
            }
            Some(Node::Button { text, on_clicked }) => {
                let resp = ui.button(text.clone());
                if resp.clicked() {
                    client.send_event(Event::Clicked {
                        id: on_clicked.to_string(),
                    });
                }
            }
            Some(Node::LeftToRightLayout {}) => {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    for child in &val.children {
                        child.draw(client, ctx, ui);
                    }
                });
                return;
            }
            Some(Node::Window { title }) => {
                let window = egui::Window::new(title.clone()).id(egui::Id::new(val.id));
                window.show(ctx, |ui| {
                    for child in &val.children {
                        child.draw(client, ctx, ui);
                    }
                });
                return;
            }
            Some(Node::ComboBox {
                label,
                selected,
                options,
                on_changed,
            }) => {
                let resp = egui::ComboBox::from_label(label.clone())
                    .selected_text(format!("{:?}", selected))
                    .show_ui(ui, |ui| {
                        for option in options {
                            ui.selectable_value(selected, option.clone(), option.clone());
                        }
                    })
                    .response;

                if resp.changed() {
                    client.send_event(Event::TextChanged {
                        id: on_changed.to_string(),
                        text: selected.to_string(),
                    });
                }
            }
            None => {}
        }

        for child in &val.children {
            child.draw(client, ctx, ui);
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

    fn draw(&self, client: &Client, ctx: &egui::Context, ui: &mut egui::Ui) -> () {
        self.root.draw(client, ctx, ui);
    }

    fn get_child(&self, id: u32) -> Option<SceneNode> {
        self.root.get_child(id)
    }
}

struct SocketListener {
    scene: Arc<Scene>,
    ctx: Context,
    client: Client,
}

impl SocketListener {
    fn new(scene: Arc<Scene>, ctx: Context, client: Client) -> Self {
        Self { scene, ctx, client }
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
                    if let Some(_) = self.scene.get_child(object_id) {
                        println!("Duplicate object ID: {}", object_id);
                    } else {
                        let parent = self.scene.get_child(parent_id);
                        if let Some(parent) = parent {
                            let new_child = SceneNode::new(object_id);
                            new_child.set_node(node);
                            parent.append(new_child);
                        }
                    }
                }
                EditCommand::ReplaceNode { object_id, node } => {
                    if let Some(sn) = self.scene.get_child(object_id) {
                        sn.set_node(node);
                    } else {
                        println!("Object {} does not exist.", object_id);
                    }
                }
            }
        }
    }

    fn listen(&self) {
        println!("listening on ws://127.0.0.1:3012/");
        ws::listen("127.0.0.1:3012", |out| {
            self.client.add_sender(out);

            move |msg: ws::Message| {
                if let Ok(text) = msg.into_text() {
                    // println!("got text: {}", text);
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
    client: Client,
}

impl Application {
    fn new(scene: Arc<Scene>) -> Application {
        Application {
            scene: scene,
            listening: false,
            client: Client::new(),
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.listening {
            let scene_clone = self.scene.clone();
            let ctx_clone = ctx.clone();
            let client_clone = self.client.clone();

            thread::spawn(move || {
                SocketListener::new(scene_clone, ctx_clone, client_clone).listen();
            });

            self.listening = true;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.scene.draw(&self.client, ctx, ui);
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
        "UIF2",
        options,
        Box::new(|_cc| Box::new(Application::new(app_scene))),
    )
}
