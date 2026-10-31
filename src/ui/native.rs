//! Standard ViewKit chrome; calculator-specific behavior stays in the model.
use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ToolbarAction {
    Modes,
    Back,
    History,
    More,
    ClearHistory,
}

#[derive(Clone, Copy)]
enum Control {
    Base,
    Bits,
    Category,
    From,
    To,
    Swap,
    Angle,
    Signed,
}
struct NativeControl {
    core: Rc<Core>,
    kind: Control,
}
impl NativeControl {
    fn with_view<R>(&self, f: impl FnOnce(&dyn View) -> R) -> R {
        match self.kind {
            Control::Base => f(&self.core.pickers.borrow().base),
            Control::Bits => f(&self.core.pickers.borrow().bits),
            Control::Category => f(&self.core.pickers.borrow().category),
            Control::From => f(&self.core.pickers.borrow().from),
            Control::To => f(&self.core.pickers.borrow().to),
            Control::Swap => f(&self.core.swap),
            Control::Angle => f(&self.core.angle),
            Control::Signed => f(&self.core.signed_control),
        }
    }
}
impl View for NativeControl {
    fn measure(&self, c: Constraints, context: &mut MeasureContext<'_>) -> Size {
        self.with_view(|view| view.measure(c, context))
    }
    fn paint(&self, bounds: Rect, context: &mut PaintContext<'_>) {
        self.with_view(|view| view.paint(bounds, context));
    }
    fn handle_event(
        &self,
        bounds: Rect,
        event: &ViewEvent,
        context: &mut EventContext<'_>,
    ) -> EventResult {
        self.with_view(|view| view.handle_event(bounds, event, context))
    }
}
impl CalculatorView {
    pub(super) fn toolbar_view(&self) -> ActionToolbar<ToolbarAction> {
        if self.history_only {
            return ActionToolbar::new(self.core.history_toolbar.clone())
                .title("History")
                .trailing(
                    ToolbarAction::ClearHistory,
                    Button::with_interaction_and_label(
                        self.core.clear_history.interaction().clone(),
                        "Clear History",
                    )
                    .on_click({
                        let pending = self.core.action.clone();
                        move || pending.set(Some(Action::ClearHistory))
                    })
                    .enabled(!self.core.model.borrow().history.is_empty()),
                );
        }
        let mode = self.current_settings().mode;
        let pending = self.core.action.clone();
        let mode_button =
            Button::with_interaction_and_label(self.core.mode_button.clone(), mode.label())
                .content(
                    HStack::new()
                        .gap(StackGap::Small)
                        .alignment(StackAlignment::Center)
                        .child(Text::label(mode.label()).single_line(true))
                        .child(
                            Icon::new(SymbolName::ChevronDown)
                                .size(Theme::current().layout.compact_icon_size),
                        ),
                )
                .accessibility_label("Calculator mode")
                .on_click(move || pending.set(Some(Action::Modes)));
        ActionToolbar::new(self.core.toolbar.clone())
            .context_key(index(mode) as u64)
            .leading(ToolbarAction::Modes, mode_button)
            .trailing(
                ToolbarAction::Back,
                Button::with_interaction_and_label(self.core.copy.interaction().clone(), "Back")
                    .accessibility_label("Delete previous character")
                    .on_click({
                        let pending = self.core.action.clone();
                        move || pending.set(Some(Action::Back))
                    }),
            )
            .trailing(
                ToolbarAction::History,
                Button::with_interaction_and_label(
                    self.core.history.interaction().clone(),
                    "History",
                )
                .on_click({
                    let pending = self.core.action.clone();
                    move || pending.set(Some(Action::History))
                }),
            )
            .trailing(
                ToolbarAction::More,
                Button::with_interaction_and_label(
                    self.core.default_mode.interaction().clone(),
                    "",
                )
                .leading_symbol(SymbolName::More)
                .accessibility_label("Calculator Actions")
                .on_click({
                    let pending = self.core.action.clone();
                    move || pending.set(Some(Action::More))
                }),
            )
    }
    pub(super) fn toolbar_anchor(
        &self,
        id: ToolbarAction,
        frames: &Frames,
        context: &mut EventContext<'_>,
    ) -> Point {
        let frame = self
            .toolbar_view()
            .action_bounds(&id, frames.toolbar, &mut context.measure_context())
            .unwrap_or(frames.toolbar);
        Point::new(frame.origin.x, frame.origin.y + frame.size.height)
    }
    pub(super) fn mode_controls(&self) -> Box<dyn View> {
        let control = |kind| NativeControl {
            core: self.core.clone(),
            kind,
        };
        match self.current_settings().mode {
            Mode::Scientific => {
                Box::new(HStack::new().child(control(Control::Angle).layout().flex_grow(1.0)))
            }
            Mode::Programmer => Box::new(
                HStack::new()
                    .gap(StackGap::Small)
                    .child(control(Control::Base).layout().flex_grow(1.0))
                    .child(control(Control::Bits).layout().flex_grow(1.0))
                    .child(control(Control::Signed)),
            ),
            Mode::Convert => Box::new(
                VStack::new()
                    .alignment(StackAlignment::Stretch)
                    .gap(StackGap::Small)
                    .child(control(Control::Category))
                    .child(
                        HStack::new()
                            .gap(StackGap::Small)
                            .child(control(Control::From).layout().flex_grow(1.0))
                            .child(control(Control::Swap))
                            .child(control(Control::To).layout().flex_grow(1.0)),
                    ),
            ),
            Mode::Standard => Box::new(HStack::new()),
        }
    }
    pub(super) fn history_content(&self) -> Scroll {
        Scroll::new(self.core.history_scroll.clone()).content(HistoryList(self.clone()))
    }
}
struct HistoryList(CalculatorView);
impl HistoryList {
    fn content(&self, theme: &Theme) -> Box<dyn View> {
        let model = self.0.core.model.borrow();
        if model.history.is_empty() {
            return Box::new(Padding::all(theme.spacing.extra_large).content(
                Text::body("Your calculations will appear here.").tone(TextTone::Secondary),
            ));
        }
        Box::new(
            List::new().rows(model.history.iter().enumerate().rev().map(|(i, entry)| {
                self.0
                    .history_row(i, entry)
                    .height(theme.layout.list_row_height)
            })),
        )
    }
}
impl View for HistoryList {
    fn measure(&self, c: Constraints, context: &mut MeasureContext<'_>) -> Size {
        self.content(context.theme).measure(c, context)
    }
    fn paint(&self, bounds: Rect, context: &mut PaintContext<'_>) {
        self.content(context.theme).paint(bounds, context);
    }
    fn handle_event(
        &self,
        bounds: Rect,
        event: &ViewEvent,
        context: &mut EventContext<'_>,
    ) -> EventResult {
        self.content(context.theme())
            .handle_event(bounds, event, context)
    }
}
