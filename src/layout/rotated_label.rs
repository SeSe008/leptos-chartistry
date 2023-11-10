use super::{
    compose::{UseLayout, VerticalLayout},
    HorizontalLayout, HorizontalOption, VerticalOption,
};
use crate::{
    bounds::Bounds, chart::Attr, debug::DebugRect, edge::Edge, projection::Projection,
    series::UseSeries, Font, Padding,
};
use leptos::*;
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
pub enum Anchor {
    Start,
    Middle,
    End,
}

#[derive(Clone, Debug)]
pub struct RotatedLabel {
    text: MaybeSignal<String>,
    anchor: MaybeSignal<Anchor>,
    font: Option<MaybeSignal<Font>>,
    padding: Option<MaybeSignal<Padding>>,
    debug: Option<MaybeSignal<bool>>,
}

pub type RotatedLabelAttr = UseRotatedLabel;

#[derive(Clone, Debug)]
pub struct UseRotatedLabel {
    text: MaybeSignal<String>,
    anchor: MaybeSignal<Anchor>,
    font: MaybeSignal<Font>,
    padding: MaybeSignal<Padding>,
    debug: MaybeSignal<bool>,
}

impl RotatedLabel {
    pub fn new(
        anchor: impl Into<MaybeSignal<Anchor>>,
        text: impl Into<MaybeSignal<String>>,
    ) -> Self {
        Self {
            text: text.into(),
            anchor: anchor.into(),
            font: None,
            padding: None,
            debug: None,
        }
    }

    pub fn start(text: impl Into<MaybeSignal<String>>) -> Self {
        Self::new(Anchor::Start, text)
    }
    pub fn middle(text: impl Into<MaybeSignal<String>>) -> Self {
        Self::new(Anchor::Middle, text)
    }
    pub fn end(text: impl Into<MaybeSignal<String>>) -> Self {
        Self::new(Anchor::End, text)
    }

    pub fn set_font(mut self, font: impl Into<MaybeSignal<Font>>) -> Self {
        self.font = Some(font.into());
        self
    }

    pub fn set_padding(mut self, padding: impl Into<MaybeSignal<Padding>>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    pub fn set_debug(mut self, debug: impl Into<MaybeSignal<bool>>) -> Self {
        self.debug = Some(debug.into());
        self
    }

    fn apply_attr(self, attr: &Attr) -> RotatedLabelAttr {
        UseRotatedLabel {
            text: self.text,
            anchor: self.anchor,
            font: self.font.unwrap_or(attr.font),
            padding: self.padding.unwrap_or(attr.padding),
            debug: self.debug.unwrap_or(attr.debug),
        }
    }
}

impl<X: 'static, Y: 'static> HorizontalLayout<X, Y> for RotatedLabel {
    fn apply_attr(self, attr: &Attr) -> Rc<dyn HorizontalOption<X, Y>> {
        Rc::new(self.apply_attr(attr))
    }
}

impl<X: 'static, Y: 'static> VerticalLayout<X, Y> for RotatedLabel {
    fn apply_attr(self, attr: &Attr) -> Rc<dyn VerticalOption<X, Y>> {
        Rc::new(self.apply_attr(attr))
    }
}

impl<X, Y> HorizontalOption<X, Y> for UseRotatedLabel {
    fn height(&self) -> Signal<f64> {
        self.size()
    }

    fn into_use(self: Rc<Self>, _: &UseSeries<X, Y>, _: Signal<f64>) -> Box<dyn UseLayout> {
        Box::new((*self).clone())
    }
}

impl<X, Y> VerticalOption<X, Y> for UseRotatedLabel {
    fn into_use(self: Rc<Self>, _: &UseSeries<X, Y>, _: Signal<f64>) -> Box<dyn UseLayout> {
        Box::new((*self).clone())
    }
}

impl UseRotatedLabel {
    pub fn size(&self) -> Signal<f64> {
        let (text, font, padding) = (self.text.clone(), self.font, self.padding);
        Signal::derive(move || {
            if text.with(|t| t.is_empty()) {
                0.0
            } else {
                with!(|font, padding| font.height() + padding.height())
            }
        })
    }
}

impl Anchor {
    fn as_svg_attr(&self) -> &'static str {
        match self {
            Anchor::Start => "start",
            Anchor::Middle => "middle",
            Anchor::End => "end",
        }
    }

    fn map_points(&self, left: f64, middle: f64, right: f64) -> f64 {
        match self {
            Anchor::Start => left,
            Anchor::Middle => middle,
            Anchor::End => right,
        }
    }

    pub fn css_justify_content(&self) -> &'static str {
        match self {
            Anchor::Start => "flex-start",
            Anchor::Middle => "center",
            Anchor::End => "flex-end",
        }
    }
}

impl UseLayout for UseRotatedLabel {
    fn width(&self) -> Signal<f64> {
        self.size()
    }

    fn render<'a>(&self, edge: Edge, bounds: Bounds, _: Signal<Projection>) -> View {
        view! { <RotatedLabel label=self.clone() edge=edge bounds=bounds /> }
    }
}

#[component]
pub(super) fn RotatedLabel(label: UseRotatedLabel, edge: Edge, bounds: Bounds) -> impl IntoView {
    let UseRotatedLabel {
        text,
        anchor,
        font,
        padding,
        debug,
    } = label;

    let content = Signal::derive(move || padding.get().apply(bounds));
    let position = Signal::derive(move || {
        let c = content.get();
        let (top, right, bottom, left) = (c.top_y(), c.right_x(), c.bottom_y(), c.left_x());
        let (centre_x, centre_y) = (c.centre_x(), c.centre_y());

        let anchor = anchor.get();
        match edge {
            Edge::Top | Edge::Bottom => (0, anchor.map_points(left, centre_x, right), centre_y),
            Edge::Left => (270, centre_x, anchor.map_points(bottom, centre_y, top)),
            // Right rotates the opposite way to Left inverting the anchor points
            Edge::Right => (90, centre_x, anchor.map_points(top, centre_y, bottom)),
        }
    });

    view! {
        <g class="_chartistry_rotated_label">
            <DebugRect label="RotatedLabel" debug=debug bounds=move || vec![bounds, content.get()] />
            <text
                x=move || position.with(|(_, x, _)| x.to_string())
                y=move || position.with(|(_, _, y)| y.to_string())
                transform=move || position.with(|(rotate, x, y)| format!("rotate({rotate}, {x}, {y})"))
                dominant-baseline="middle"
                text-anchor=move || anchor.get().as_svg_attr()
                font-family=move || font.get().svg_family()
                font-size=move || font.get().svg_size()>
                {text}
            </text>
        </g>
    }
}
