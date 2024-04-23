use std::sync::Arc;

use iced::{Alignment, Background, Color, Command, Element, Length, Vector};
use iced::widget::{Button, Column, Container, container, pick_list, Row, Space, Text};
use iced::widget::image::Handle as ImageHandle;
use iced_aw::{BOOTSTRAP_FONT, BootstrapIcon};
use iced_aw::graphics::icons::icon_to_char;
use image::RgbaImage;

use crate::fairplay::{Fairplay, Message};
use crate::interface::components::{SelectedButtonStyle, TransparentButtonStyle, with_spinner};
use crate::interface::editing_components::modifier_options;
use crate::interface::graph::graph;
use crate::interface::View;
use crate::models::modifier::{BoxBlurOptions, GrayscaleOptions, LightnessCorrectionOptions, MedianBlurOptions, Modifier, NegativeOptions, SobelOptions, ThresholdingOptions, UnsharpMaskingOptions};
use crate::services;
use crate::interface::graph_editor;
use crate::interface::graph_editor::Event;
use crate::models::node::Node;

pub struct EditingView {
    pub(crate) image: Arc<RgbaImage>,
    pub(crate) handle: ImageHandle,

    pub(crate) loading: bool,
    pub(crate) modifiers: Vec<Modifier>,
    pub(crate) selected_modifier: Option<(usize, Modifier)>,

    pub(crate) nodes: Vec<Node>,
    pub(crate) scaling: f32,
    pub(crate) translation: Vector
}

impl View for EditingView {
    fn update(app: &mut Fairplay, message: Message) -> Command<Message> {
        let Fairplay::Editing(state) = app else { panic!("Invalid call!") };
        match message {
            Message::ModifierAdded(modifier) => {
                state.loading = true;
                state.modifiers.push(modifier.clone());
                state.selected_modifier = Some((state.modifiers.len() - 1, modifier));
                return Command::perform(services::image::apply(state.image.clone(), state.modifiers.clone()), |r| Message::ImageModified(r));
            }
            Message::ModifierRemoved(idx) => {
                state.loading = true;
                if let Some((i, _)) = &state.selected_modifier {
                    if *i == idx {
                        state.selected_modifier = None;
                    }
                }
                state.modifiers.remove(idx);
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
                let selected = state.selected_modifier.clone().unwrap();
                state.modifiers[selected.0] = selected.1;
                return Command::perform(services::image::apply(state.image.clone(), state.modifiers.clone()), |r| Message::ImageModified(r));
            }
            Message::ModifierSelected(idx, modifier) => {
                if let Some((i, _)) = &state.selected_modifier {
                    if *i == idx {
                        state.selected_modifier = None;
                    } else {
                        state.selected_modifier = Some((idx, modifier));
                    }
                } else {
                    state.selected_modifier = Some((idx, modifier));
                }
            }
            Message::Graph(event) => match event {
                Event::NodeMoved { index, offset } => {
                    state.nodes[index].offset = offset;
                }
                Event::Scaled(scaling, translation) => {
                    state.scaling = scaling;
                    state.translation = translation;
                }
                Event::Translated(translation) => {
                    state.translation = translation;
                }
            }
            _ => { panic!("Invalid message") }
        };

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let dropdown = container(
            pick_list(
                vec![
                    Modifier::Negative(NegativeOptions::default()),
                    Modifier::Thresholding(ThresholdingOptions::default()),
                    Modifier::Grayscale(GrayscaleOptions::default()),
                    Modifier::LightnessCorrection(LightnessCorrectionOptions::default()),
                    Modifier::BoxBlur(BoxBlurOptions::default()),
                    Modifier::MedianBlur(MedianBlurOptions::default()),
                    Modifier::Sobel(SobelOptions::default()),
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

        let graph_editor = graph(&self.nodes, self.scaling, self.translation);


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
            .push(graph_editor)
            .push(image)
            .push(panel)
            .into();

        if self.loading {
            with_spinner(row)
        } else {
            row
        }
    }
}