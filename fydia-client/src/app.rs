#![allow(dead_code)]
#![allow(unused_variables)]

use crossbeam_channel::Receiver;
use eframe::{egui, epi};
use egui::{Align, Color32, Id, Label, ScrollArea, Style, TextStyle, Ui, Vec2};
use futures_util::StreamExt;
use fydia_struct::{
    channel::ChannelId,
    instance::Instance,
    messages::{Message, SqlDate},
    user::User,
};
use tokio_tungstenite::connect_async;

pub enum State {
    Login,
    Token(String),
}
#[derive(Clone)]
struct ReadChannel(pub Option<Receiver<tokio_tungstenite::tungstenite::Message>>);
impl ReadChannel {
    fn set(&mut self, receive: Receiver<tokio_tungstenite::tungstenite::Message>) {
        self.0 = Some(receive);
    }
}
pub struct FydiaApp {
    state: State,
    read_channel: ReadChannel,
    label: Vec<Message>,
    msg: String,
}

impl FydiaApp {
    pub fn push(&mut self, msg: Message) {
        self.label.push(msg);
    }
}

impl Default for FydiaApp {
    fn default() -> Self {
        let mut vec = Vec::new();
        let mut user = User::default();
        user.name = String::from("Rheydskey");
        user.instance = Instance::new(
            fydia_struct::instance::Protocol::HTTP,
            "rheydskey.studio".to_string(),
            8080,
        );
        for _ in 0..20 {
            vec.push(Message::new(
                "String".to_string(),
                fydia_struct::messages::MessageType::TEXT,
                false,
                SqlDate::now(),
                user.clone(),
                ChannelId::new(String::from("02580")),
            ))
        }
        Self {
            state: State::Token(String::from("YvxxiH8l8S3d7y5n73xZFbgNuSjYur")),
            label: Vec::new(),
            msg: String::new(),
            read_channel: ReadChannel(None),
        }
    }
}

impl epi::App for FydiaApp {
    fn name(&self) -> &str {
        "Fydia App"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let Self {
            label,
            msg,
            state,
            read_channel,
        } = self;
        let (writer, reader) =
            crossbeam_channel::unbounded::<tokio_tungstenite::tungstenite::Message>();
        if read_channel.0.is_none() {
            tokio::spawn(async move {
                let url =
                    "ws://127.0.0.1:8080/api/user/websocket?token=YvxxiH8l8S3d7y5n73xZFbgNuSjYur";
                let (mut connection, _) = connect_async(url).await.expect("Error");
                while let Some(msg) = connection.next().await {
                    writer.clone().send(msg.expect("Error")).expect("Error");
                }
            });

            read_channel.set(reader);
        }
        //let channel = read_channel.0.unwrap();
        /*tokio::spawn(async {
            while let Ok(e) = channel.clone().recv() {
                match e {
                    tokio_tungstenite::tungstenite::Message::Text(e) => {
                        let message = serde_json::from_str::<Message>(e.as_str()).unwrap();
                        label.push(message)
                    }
                    tokio_tungstenite::tungstenite::Message::Binary(_) => todo!(),
                    tokio_tungstenite::tungstenite::Message::Ping(_) => todo!(),
                    tokio_tungstenite::tungstenite::Message::Pong(_) => todo!(),
                    tokio_tungstenite::tungstenite::Message::Close(_) => todo!(),
                }
            }
        });*/
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading(format!("Fydia - {}", env!("CARGO_PKG_VERSION")));

                ui.horizontal(|ui| {
                    ui.columns(1, |e| {
                        for _ in 0..20 {
                            e[0].label("# - Channel1");
                        }
                    });
                });
                ui.separator();

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add(
                        egui::Hyperlink::new("https://github.com/emilk/egui/")
                            .text("powered by egui"),
                    );
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Welcome to fydia");

            ui.group(|ui| {
                ui.set_max_height(ui.available_size().y - 25.0);
                ScrollArea::auto_sized().show(ui, |ui| {
                    let lock = label.iter();
                    for i in lock {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                let mut style = Style::default();
                                style.spacing.item_spacing = Vec2::new(0., 0.);
                                ui.set_style(style);
                                let textstyle = TextStyle::Body;
                                ui.add(
                                    Label::new(i.author_id.name.clone())
                                        .text_style(textstyle)
                                        .text_color(Color32::from_rgb(214, 214, 214)),
                                );
                                ui.add(Label::new(format!(
                                    "@{}",
                                    i.author_id.instance.clone().domain
                                )));
                            });
                            ui.add(Label::new(i.content.clone()).text_color(Color32::WHITE));
                        });
                        ui.add_space(2.)
                    }
                })
            });
            ui.with_layout(egui::Layout::bottom_up(Align::RIGHT), |ui: &mut Ui| {
                ui.horizontal(|ui| {
                    if ui.button("Envoyer").clicked() {
                        println!("Salue")
                    };
                    let text_edit_size = ui.available_size_before_wrap_finite();
                    let text = egui::TextEdit::singleline(msg)
                        .id(Id::new("Send_Message"))
                        .hint_text("Envoyer un message".to_string());
                    ScrollArea::auto_sized().show(ui, |ui| ui.add_sized(text_edit_size, text));
                });
            });
        });
    }
}
