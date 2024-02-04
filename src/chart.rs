use crate::{
    aspect_ratio::KnownAspectRatio,
    debug::DebugRect,
    inner::InnerLayout,
    layout::{EdgeLayout, HorizontalVec, Layout, VerticalVec},
    overlay::tooltip::Tooltip,
    projection::Projection,
    series::{RenderData, UseData},
    state::{PreState, State},
    use_watched_node::{use_watched_node, UseWatchedNode},
    AspectRatio, Padding, Series, Tick,
};
use leptos::{html::Div, *};

pub const FONT_HEIGHT: f64 = 16.0;
pub const FONT_WIDTH: f64 = 10.0;

#[component]
pub fn Chart<T: 'static, X: Tick, Y: Tick>(
    #[prop(into)] aspect_ratio: MaybeSignal<AspectRatio>,
    #[prop(into, optional)] font_height: Option<MaybeSignal<f64>>,
    #[prop(into, optional)] font_width: Option<MaybeSignal<f64>>,
    #[prop(into, optional)] debug: MaybeSignal<bool>,
    #[prop(into, optional)] padding: Option<MaybeSignal<Padding>>,

    #[prop(into, optional)] mut top: HorizontalVec<X>,
    #[prop(into, optional)] right: VerticalVec<Y>,
    #[prop(into, optional)] bottom: HorizontalVec<X>,
    #[prop(into, optional)] mut left: VerticalVec<Y>,

    #[prop(into, optional)] inner: Vec<InnerLayout<X, Y>>,
    #[prop(into, optional)] tooltip: Option<Tooltip<X, Y>>,

    #[prop(into)] series: Series<T, X, Y>,
    #[prop(into)] data: Signal<Vec<T>>,
) -> impl IntoView {
    let root = create_node_ref::<Div>();
    let watch = use_watched_node(root);

    // Aspect ratio signal
    let have_dimensions = create_memo(move |_| watch.bounds.get().is_some());
    let width = create_memo(move |_| watch.bounds.get().unwrap_or_default().width());
    let height = create_memo(move |_| watch.bounds.get().unwrap_or_default().height());
    let calc = create_memo(move |_| aspect_ratio.get().into_known(width, height));

    let debug = create_memo(move |_| debug.get());
    let font_height = create_memo(move |_| font_height.map(|f| f.get()).unwrap_or(FONT_HEIGHT));
    let font_width = create_memo(move |_| font_width.map(|f| f.get()).unwrap_or(FONT_WIDTH));
    let padding = create_memo(move |_| {
        padding
            .map(|p| p.get())
            .unwrap_or_else(move || Padding::from(font_width.get()))
    });

    // Edges are added top to bottom, left to right. Layout compoeses inside out:
    top.reverse();
    left.reverse();

    // Build data
    let data = UseData::new(series, data);
    let pre = PreState::new(debug.into(), font_height, font_width, padding.into(), data);

    view! {
        <div class="_chartistry" node_ref=root style="width: fit-content; height: fit-content; overflow: visible;">
            <DebugRect label="Chart" debug=debug />
            <Show when=move || have_dimensions.get() fallback=|| view!(<p>"Loading..."</p>)>
                <RenderChart
                    watch=watch.clone()
                    pre_state=pre.clone()
                    aspect_ratio=calc
                    top=top.as_slice()
                    right=right.as_slice()
                    bottom=bottom.as_slice()
                    left=left.as_slice()
                    inner=inner.clone()
                    tooltip=tooltip.clone()
                />
            </Show>
        </div>
    }
}

#[component]
fn RenderChart<'a, X: Tick, Y: Tick>(
    watch: UseWatchedNode,
    pre_state: PreState<X, Y>,
    aspect_ratio: Memo<KnownAspectRatio>,
    top: &'a [EdgeLayout<X>],
    right: &'a [EdgeLayout<Y>],
    bottom: &'a [EdgeLayout<X>],
    left: &'a [EdgeLayout<Y>],
    inner: Vec<InnerLayout<X, Y>>,
    tooltip: Option<Tooltip<X, Y>>,
) -> impl IntoView {
    let debug = pre_state.debug;

    // Compose edges
    let (layout, edges) = Layout::compose(top, right, bottom, left, aspect_ratio, &pre_state);

    // Finalise state
    let projection = {
        let inner = layout.inner;
        let position_range = pre_state.data.position_range;
        create_memo(move |_| Projection::new(inner.get(), position_range.get())).into()
    };
    let state = State::new(pre_state, &watch, layout, projection);

    // Render edges
    let edges = edges
        .into_iter()
        .map(|r| r.render(state.clone()))
        .collect_view();

    // Inner
    let inner = inner
        .into_iter()
        .map(|opt| opt.into_use(&state).render(state.clone()))
        .collect_view();

    let outer = state.layout.outer;
    view! {
        <svg
            width=move || format!("{}px", outer.get().width())
            height=move || format!("{}px", outer.get().height())
            viewBox=move || with!(|outer| format!("0 0 {} {}", outer.width(), outer.height()))
            style="display: block; overflow: visible;">
            <DebugRect label="RenderChart" debug=debug bounds=vec![outer.into()] />
            {inner}
            {edges}
            <RenderData state=state.clone() />
        </svg>
        {tooltip.map(|tooltip| view! {
            <Tooltip tooltip=tooltip state=state />
        })}
    }
}
