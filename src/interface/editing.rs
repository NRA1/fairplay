use std::io::{Cursor, Read};
use std::sync::{Arc, Mutex};

use iced::{Alignment, Application, Background, Color, Command, Element, Length};
use iced::futures::AsyncWriteExt;
use iced::widget::{Button, button, Column, Container, container, pick_list, Row, Space, Text};
use iced::widget::image::Handle as ImageHandle;
use iced_aw::{BOOTSTRAP_FONT, BootstrapIcon};
use iced_aw::graphics::icons::icon_to_char;
use image::{ImageFormat, RgbaImage};
use image::io::Reader as ImageReader;
use once_cell::sync::Lazy;
use rfd::AsyncFileDialog;
use undo::Record;

use crate::fairplay::{Fairplay, Message};
use crate::interface::components::{SelectedButtonStyle, TransparentButtonStyle, with_spinner};
use crate::interface::editing_components::modifier_options;
use crate::interface::View;
use crate::models::history::{Action, ModifierAdded, ModifierOptionsApplied, ModifierRemoved, ModifierSelected};
use crate::models::modifier::{BoxBlurOptions, ChannelOptions, GaussianBlurOptions, GrayscaleOptions, LightnessCorrectionOptions, MedianBlurOptions, Modifier, NegativeOptions, SobelOptions, ThresholdingOptions, UnsharpMaskingOptions};
use crate::services;

static RECORD: Lazy<Mutex<Record<Action>>> = Lazy::new(|| {
    Mutex::new(Record::new())
});

pub struct EditingView {
    pub(crate) image: Arc<RgbaImage>,
    pub(crate) handle: ImageHandle,

    pub(crate) loading: bool,
    pub(crate) modifiers: Vec<Modifier>,
    pub(crate) selected_modifier: Option<(usize, Modifier)>,
}

impl EditingView {
    pub fn new(img: RgbaImage) -> Self {
        EditingView {
            handle: ImageHandle::from_pixels(img.width(), img.height(), img.to_vec()),
            image: Arc::new(img),
            loading: false,
            modifiers: vec![],
            selected_modifier: None,
        }
    }
}

impl View for EditingView {
    fn update(app: &mut Fairplay, message: Message) -> Command<Message> {
        let Fairplay::Editing(state) = app else { panic!("Invalid call!") };
        match message {
            Message::ModifierAdded(modifier) => {
                state.loading = true;
                let r = RECORD.lock();
                if let Ok(mut rrr) = r {
                    rrr.apply(state, Action::ModifierAdded(ModifierAdded::new(modifier)));
                } else {
                    println!("Failed to acquire lock");
                }
                return Command::perform(services::image::apply(state.image.clone(), state.modifiers.clone()), |r| Message::ImageModified(r));
            }
            Message::ModifierRemoved(idx) => {
                state.loading = true;
                RECORD.lock().unwrap().apply(state, Action::ModifierRemoved(ModifierRemoved::new(idx)));
                return Command::perform(services::image::apply(state.image.clone(), state.modifiers.clone()), |r| Message::ImageModified(r));
            }
            Message::ImageModified(image) => {
                state.handle = ImageHandle::from_pixels(image.width(), image.height(), image.into_vec());
                state.loading = false;
            }
            Message::ModifierOptionsChanged(modifier) => {
                state.selected_modifier = Some((state.selected_modifier.clone().unwrap().0, modifier))
            }
            Message::ModifierOptionsApplied => {
                state.loading = true;
                RECORD.lock().unwrap().apply(state, Action::ModifierOptionsApplied(ModifierOptionsApplied::new()));
                return Command::perform(services::image::apply(state.image.clone(), state.modifiers.clone()), |r| Message::ImageModified(r));
            }
            Message::ModifierSelected(idx, modifier) => {
                RECORD.lock().unwrap().apply(state, Action::ModifierSelected(ModifierSelected::new(idx, modifier)))
            }
            Message::Undo => {
                RECORD.lock().unwrap().undo(state);
                return Command::perform(services::image::apply(state.image.clone(), state.modifiers.clone()), |r| Message::ImageModified(r));
            }
            Message::Redo => {
                RECORD.lock().unwrap().redo(state);
                return Command::perform(services::image::apply(state.image.clone(), state.modifiers.clone()), |r| Message::ImageModified(r));
            }
            Message::OpenPicker => {
                state.loading = true;

                return Command::perform(async {
                    let file = AsyncFileDialog::new()
                        .add_filter("image", &["png", "jpg"])
                        .pick_file()
                        .await;
                    file.unwrap().read().await
                }, |data| {
                    let img = ImageReader::new(Cursor::new(data))
                        .with_guessed_format()
                        .unwrap()
                        .decode()
                        .unwrap()
                        .into_rgba8();

                    Message::Open(img)
                }
                )
            }
            Message::Save => {
                let image = state.image.clone();
                let modifiers = state.modifiers.clone();
                return Command::perform(async {
                    let mut handle = AsyncFileDialog::new()
                        .set_file_name("edited.png")
                        .save_file()
                        .await;
                    if let Some(handle) = handle {
                        let img = services::image::apply(image, modifiers).await;
                        let mut mem = Cursor::new(Vec::<u8>::new());

                        #[cfg(not(target_arch = "wasm32"))]
                        let format = ImageFormat::from_path(handle.path()).unwrap();
                        #[cfg(target_arch = "wasm32")]
                        let format = ImageFormat::Png;

                        img.write_to(&mut mem, format).expect("Error writing to memory buffer");
                        handle.write(mem.get_ref()).await.expect("Error saving!");
                    }
                    ()
                }, |_| Message::Saved);
            }
            Message::Open(data) => {
                *app = Fairplay::Editing(EditingView::new(data));
            }
            Message::Saved => { }
            _ => { panic!("Invalid message") }
        };

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let dropdown =
        container(
            pick_list(
                vec![
                    Modifier::Negative(NegativeOptions::default()),
                    Modifier::Thresholding(ThresholdingOptions::default()),
                    Modifier::Grayscale(GrayscaleOptions::default()),
                    Modifier::Channels(ChannelOptions::default()),
                    Modifier::LightnessCorrection(LightnessCorrectionOptions::default()),
                    Modifier::BoxBlur(BoxBlurOptions::default()),
                    Modifier::GaussianBlur(GaussianBlurOptions::default()),
                    Modifier::MedianBlur(MedianBlurOptions::default()),
                    Modifier::Sobel(SobelOptions::default()),
                    Modifier::Laplace,
                    Modifier::Sharpening,
                    Modifier::UnsharpMasking(UnsharpMaskingOptions::default()),
                ],
                None::<Modifier>,
                |modifier: Modifier| {
                    Message::ModifierAdded(modifier)
                }
            )
                .placeholder("Add a modifier")
                .width(Length::Fill)
        )
            .width(Length::Fill)
            .padding(10);


        let mut modifiers = Column::new();

        let selected_mod_idx = if let Some((i, _)) = &self.selected_modifier {
            Some(*i)
        } else { None };

        for (i, modifier) in self.modifiers.iter().enumerate() {
            let mut mod_btn = Button::new(
                Row::new()
                    .push(
                        Text::new(format!("{}", modifier))
                            .width(Length::Fill)
                    ).push(
                    Button::new(Text::new(String::from(icon_to_char(BootstrapIcon::X))).font(BOOTSTRAP_FONT))
                        .on_press(Message::ModifierRemoved(i))
                        .style(iced::theme::Button::Custom(Box::new(TransparentButtonStyle)))
                )
                    .align_items(Alignment::Center)
                    .padding([0, 10])
            )
                .width(Length::Fill)
                .on_press(Message::ModifierSelected(i, modifier.clone()));

            if selected_mod_idx.is_some_and(|idx| idx == i) {
                mod_btn = mod_btn.style(iced::theme::Button::Custom(Box::new(SelectedButtonStyle)))
            } else {
                mod_btn = mod_btn.style(iced::theme::Button::Custom(Box::new(TransparentButtonStyle)))
            }

            modifiers = modifiers.push(mod_btn);
        }

        let options = if let Some(modifier) = &self.selected_modifier {
            Some(modifier_options(&modifier.1))
        } else {
            None
        };

        let image = iced::widget::image::viewer(self.handle.clone())
            .min_scale(0.5)
            .width(Length::FillPortion(4))
            .height(Length::Fill);

        let menu = Container::new(
            Row::new()
                .push(
                    button("Open").on_press(Message::OpenPicker)
                )
                .push(
                    button("Save").on_press(Message::Save)
                )
                .push(
                    button("Undo").on_press_maybe(
                        if RECORD.lock().unwrap().can_undo() {
                            Some(Message::Undo)
                        } else { None }
                    )
                )
                .push(
                    button("Redo").on_press_maybe(
                        if RECORD.lock().unwrap().can_redo() {
                            Some(Message::Redo)
                        } else { None }
                    )
                )
                .width(Length::Fill)
                .align_items(Alignment::Start)
                .spacing(10)
        ).style(container::Appearance::default()
            .with_background(Background::from(Color::new(0.0, 0.0, 0.0, 0.3)))
        );

        let panel = Container::new(
            Column::new()
                .push(dropdown)
                .push(modifiers)
                .push(Space::new(Length::Fill, Length::Fill))
                .push_maybe(options)
                .spacing(10)
                .width(Length::FillPortion(1))
                .align_items(Alignment::Start)
                .height(Length::Fill)
        ).style(container::Appearance::default()
            .with_background(Background::from(Color::new(0.0, 0.0, 0.0, 0.3)))
        );

        let row = Row::new()
            .push(image)
            .push(panel)
            .height(Length::Fill);

        let column = Column::new()
            .push(menu)
            .push(row)
            .into();

        if self.loading {
            with_spinner(column)
        } else {
            column
        }
    }
}